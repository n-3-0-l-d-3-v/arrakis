use std::path::Path;

use capability::{CapError, Kernel, Rights};
use channel::FaultProfile;
use isa::{Block, Instruction, Opcode, Program};
use kv::Command;
use object::{sha256, ObjectId};
use repo::{Files, Repo};
use row::Value;
use sim::{Cluster, ClusterConfig, Faults, StorageKind};
use sync::{fetch, push, DistransRemote, NetConfig, NetStats};

use crate::trace::Trace;

/// Anything that goes wrong in any layer, with its message. (No `Display`, so
/// the blanket `From` below does not collide with the reflexive one.)
#[derive(Debug)]
pub struct PipelineError(pub String);

impl<E: std::fmt::Display> From<E> for PipelineError {
    fn from(e: E) -> Self {
        PipelineError(e.to_string())
    }
}

type R<T> = Result<T, PipelineError>;

fn fail<T>(msg: impl Into<String>) -> R<T> {
    Err(PipelineError(msg.into()))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn dir_bytes(dir: &Path) -> u64 {
    std::fs::read_dir(dir)
        .map(|it| {
            it.filter_map(Result::ok)
                .map(|e| e.metadata().map_or(0, |m| m.len()))
                .sum()
        })
        .unwrap_or(0)
}

/// Compiles source to a self-contained, directly runnable mentat program,
/// serialized as JSON. (The same driver-block wrapping `chakobsac build`
/// performs: set the stack pointer, call `main`, halt.)
pub fn compile_to_artifact(source: &str) -> R<Vec<u8>> {
    let module = parser::parse(source)?;
    ir::validate_module(&module)?;
    if module.function("main").map(|f| f.params.len()) != Some(0) {
        return fail("the source must define a zero-argument `fn main() -> i64`");
    }
    let compiled = codegen::compile_module(&module)?;
    let main_entry = compiled.function_entry["main"];
    let mut blocks = compiled.program.blocks;
    let driver = blocks.len();
    blocks.push(Block {
        label: "driver".to_string(),
        instructions: vec![
            Instruction::new(
                Opcode::LoadI,
                codegen::regalloc::STACK_PTR,
                0,
                0,
                codegen::STACK_BASE as i32,
            ),
            Instruction::new(Opcode::Call, 0, 0, 0, main_entry as i32),
        ],
    });
    blocks.push(Block {
        label: "halt".to_string(),
        instructions: vec![Instruction::new(Opcode::Halt, 0, 0, 0, 0)],
    });
    let program = Program {
        blocks,
        entry: driver,
    };
    program.validate()?;
    Ok(serde_json::to_vec(&program)?)
}

/// Runs a compiled artifact on mentat and returns `main`'s result.
pub fn run_artifact(artifact: &[u8]) -> R<i64> {
    let program: Program = serde_json::from_slice(artifact)?;
    program.validate()?;
    let mut machine = vm::Vm::new(program);
    machine.run()?;
    Ok(machine.regs.get(codegen::regalloc::RET_REG) as i64)
}

/// Deliberate sabotage, so tests can show each verification layer is
/// load-bearing: a run with sabotage must fail rather than execute anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tamper {
    None,
    /// Flip one bit of the artifact's stored blob on the server (after the push).
    ServerBlob,
    /// Make the SQL registry claim a different artifact hash.
    RegistryHash,
}

/// The operation to perform and the hostility to perform it under.
#[derive(Debug, Clone)]
pub struct Scenario {
    /// Seeds every simulated network and the consensus cluster.
    pub seed: u64,
    /// Datagram loss rate on both repository transfers and inside the cluster
    /// (plus duplication, reordering, corruption and truncation when > 0).
    pub loss: f64,
    /// Crash and partition consensus nodes while the release is published.
    pub chaos: bool,
    pub source: String,
    pub version: String,
    pub tamper: Tamper,
}

impl Scenario {
    pub fn new(seed: u64) -> Self {
        Scenario {
            tamper: Tamper::None,
            seed,
            loss: 0.0,
            chaos: false,
            source: crate::DEMO_SOURCE.to_string(),
            version: "v1".to_string(),
        }
    }

    pub fn hostile(seed: u64, loss: f64) -> Self {
        Scenario {
            loss,
            chaos: true,
            ..Scenario::new(seed)
        }
    }
}

/// Everything a run produced, for assertions and for display.
#[derive(Debug, Clone)]
pub struct Outcome {
    pub trace: Trace,
    pub publisher_result: i64,
    pub consumer_result: i64,
    pub commit: ObjectId,
    pub artifact_sha: String,
    pub artifact_bytes: usize,
    pub push_stats: NetStats,
    pub fetch_stats: NetStats,
    pub consensus_crashes: u64,
    pub consensus_partition_drops: u64,
    pub consensus_ticks: u64,
    pub registry_rows: Vec<Vec<Value>>,
    /// True if, after the run, the runner's capability was revoked and stopped working.
    pub capability_revoked: bool,
}

fn net_cfg(seed: u64, loss: f64) -> NetConfig {
    if loss > 0.0 {
        NetConfig::hostile(seed, loss)
    } else {
        NetConfig::clean(seed)
    }
}

fn describe(s: &NetStats) -> String {
    format!(
        "{} ticks, {} calls, {} datagrams sent, {} dropped, {} duplicated, {} corrupted",
        s.ticks,
        s.calls,
        s.client_to_server.sent + s.server_to_client.sent,
        s.client_to_server.dropped + s.server_to_client.dropped,
        s.client_to_server.duplicated + s.server_to_client.duplicated,
        s.client_to_server.corrupted + s.server_to_client.corrupted,
    )
}

impl Scenario {
    /// Performs the whole operation under `work` (created if needed) and
    /// verifies every hop. Any failed check is an error, never a silent pass.
    pub fn run(&self, work: &Path) -> R<Outcome> {
        let mut t = Trace::default();

        // 1. chakobsa: source -> bytecode.
        let artifact = compile_to_artifact(&self.source)?;
        t.note(
            "chakobsa",
            format!(
                "compiled {} bytes of source to {} bytes of mentat bytecode (typed SSA built directly, no AST)",
                self.source.len(),
                artifact.len()
            ),
        );

        // 2. mentat: the reference result.
        let publisher_result = run_artifact(&artifact)?;
        t.note(
            "mentat",
            format!("publisher ran the bytecode locally: main() = {publisher_result}"),
        );

        // 3. ghola on sietch: commit source + artifact + manifest.
        let sha = hex(&sha256(&artifact));
        let manifest = format!(
            "version={}\nartifact_sha256={}\nartifact_bytes={}\n",
            self.version,
            sha,
            artifact.len()
        );
        let mut files = Files::new();
        files.insert(b"src/main.ck".to_vec(), self.source.clone().into_bytes());
        files.insert(b"build/main.ckp".to_vec(), artifact.clone());
        files.insert(b"MANIFEST".to_vec(), manifest.into_bytes());
        let publisher_dir = work.join("publisher");
        let mut publisher = Repo::open(&publisher_dir)?;
        let tree = publisher.write_tree(&files)?;
        let commit = publisher.write_commit(
            tree,
            vec![],
            "publisher",
            &format!("release {}", self.version),
            1,
        )?;
        publisher.set_ref("main", commit)?;
        let stats = publisher.stats();
        t.note(
            "ghola",
            format!(
                "committed release {} as {} ({} content-addressed objects, {} bytes)",
                self.version,
                &commit.to_hex()[..12],
                stats.objects,
                stats.bytes
            ),
        );
        t.note(
            "sietch",
            format!(
                "the repository is an append-only checksummed log: {} bytes on disk, no byte overwritten",
                dir_bytes(&publisher_dir)
            ),
        );

        // 4. choam on sietch: index the release in SQL.
        let engine = engine::Engine::open(work.join("registry"))?;
        let mut sql = engine.session();
        sql.execute(
            "CREATE TABLE releases (version TEXT PRIMARY KEY, commit_id TEXT NOT NULL, \
             artifact_bytes INTEGER NOT NULL, artifact_sha TEXT NOT NULL)",
        )?;
        sql.execute(&format!(
            "INSERT INTO releases VALUES ('{}', '{}', {}, '{}')",
            self.version,
            commit.to_hex(),
            artifact.len(),
            sha
        ))?;
        if self.tamper == Tamper::RegistryHash {
            sql.execute(&format!(
                "UPDATE releases SET artifact_sha = '{}' WHERE version = '{}'",
                "0".repeat(64),
                self.version
            ))?;
        }
        let registry_rows = sql
            .execute(&format!(
                "SELECT version, commit_id, artifact_bytes, artifact_sha FROM releases WHERE version = '{}'",
                self.version
            ))?
            .rows;
        if registry_rows.len() != 1 {
            return fail("the registry did not return exactly the release just inserted");
        }
        t.note(
            "choam",
            format!(
                "indexed the release in SQL and read it back: {:?}",
                registry_rows[0][..3].to_vec()
            ),
        );

        // 5. distrans: publish to the server over a hostile network.
        let mut server = Repo::open(work.join("server"))?;
        let push_stats = {
            let mut net = DistransRemote::new(&mut server, net_cfg(self.seed, self.loss));
            push(&mut publisher, &mut net, "main", false)?;
            let s = net.stats();
            if net.executions() != s.calls {
                return fail("a request executed more than once on the server");
            }
            s
        };
        t.note(
            "distrans",
            format!(
                "pushed the repository to the server: {}",
                describe(&push_stats)
            ),
        );
        if server.get_ref("main") != Some(commit) {
            return fail("the server's main is not the published commit");
        }
        if self.tamper == Tamper::ServerBlob {
            drop(server);
            let blob = object::Object::Blob(artifact.clone()).encode()?;
            let mut key = vec![b'o'];
            key.extend(ObjectId::of(&blob).0);
            let mut raw = storage::Store::open(work.join("server"))?;
            let mut bad = raw
                .get(&key)
                .ok_or("the artifact blob is not on the server")?;
            let mid = bad.len() / 2;
            bad[mid] ^= 1;
            raw.put(key, bad)?;
            drop(raw);
            server = Repo::open(work.join("server"))?;
            t.note(
                "sietch",
                "SABOTAGE: flipped one bit of the artifact blob stored on the server",
            );
        }

        // 6. landsraad: consensus on the published head, under fire.
        let cluster_dir = work.join("consensus");
        let mut ccfg = ClusterConfig::new(3, self.seed);
        ccfg.storage = StorageKind::Sietch(cluster_dir);
        if self.loss > 0.0 {
            ccfg.profile = FaultProfile {
                loss: self.loss,
                duplication: 0.1,
                reorder_max_delay: 12,
                corruption: 0.03,
                truncation: 0.03,
                base_delay: 2,
            };
        }
        if self.chaos {
            ccfg.faults = Faults {
                crash: 0.004,
                restart: 0.02,
                partition: 0.004,
                heal: 0.02,
                crash_after_event: 0.001,
                propose: 0.0,
            };
        }
        let mut cluster = Cluster::new(ccfg);
        let head_key = b"release/latest".to_vec();
        let head_cmd = Command::Put {
            key: head_key.clone(),
            value: commit.to_hex().into_bytes(),
        }
        .encode();
        let committed_everywhere = |c: &Cluster| {
            (0..c.node_count()).all(|i| {
                c.node(i).is_some_and(|n| {
                    n.log()[..n.commit_index() as usize]
                        .iter()
                        .any(|e| e.command == head_cmd)
                })
            })
        };
        let chaos_ticks = if self.chaos { 2500 } else { 0 };
        let mut ticks = 0u64;
        let mut quiesced = false;
        loop {
            if !quiesced && ticks >= chaos_ticks {
                cluster.quiesce()?;
                quiesced = true;
            }
            let in_a_log = (0..cluster.node_count())
                .filter_map(|i| cluster.node(i))
                .any(|n| n.log().iter().any(|e| e.command == head_cmd));
            if !in_a_log && ticks.is_multiple_of(50) {
                let _ = cluster.propose(head_cmd.clone());
            }
            cluster.step()?;
            ticks += 1;
            if quiesced && committed_everywhere(&cluster) {
                break;
            }
            if ticks > 60_000 {
                return fail("the cluster did not commit the release head within 60,000 ticks");
            }
        }
        let cstats = cluster.stats();
        let mut heads = Vec::new();
        for i in 0..cluster.node_count() {
            let n = cluster.node(i).ok_or("a node is down after quiesce")?;
            let store = kv::replay(&n.log()[..n.commit_index() as usize]);
            heads.push(store.get(&head_key).map(<[u8]>::to_vec));
        }
        if heads
            .iter()
            .any(|h| h.as_deref() != Some(commit.to_hex().as_bytes()))
        {
            return fail("the consensus nodes do not all agree on the published head");
        }
        t.note(
            "landsraad",
            format!(
                "3 nodes (storage on sietch) agreed on release/latest = {} after {} ticks, {} crashes, {} restarts, {} leader terms{}",
                &commit.to_hex()[..12],
                ticks,
                cstats.crashes,
                cstats.restarts,
                cstats.terms_with_a_leader,
                if self.chaos { ", under partitions" } else { "" }
            ),
        );

        // 7. distrans + ghola: the consumer fetches and verifies.
        let mut consumer = Repo::open(work.join("consumer"))?;
        let fetch_stats = {
            let mut net = DistransRemote::new(&mut server, net_cfg(self.seed ^ 0x55, self.loss));
            fetch(&mut consumer, &mut net, "origin")?;
            let s = net.stats();
            if net.executions() != s.calls {
                return fail("a request executed more than once on the server");
            }
            s
        };
        let agreed =
            ObjectId::from_hex(&String::from_utf8_lossy(heads[0].as_deref().unwrap_or(b"")))
                .ok_or("the agreed head is not a valid commit id")?;
        if consumer.get_ref("remotes/origin/main") != Some(agreed) || !consumer.is_complete(&agreed)
        {
            return fail("the consumer does not hold the agreed head's complete history");
        }
        let got = consumer.read_tree(&consumer.commit_of(&agreed)?.tree)?;
        let artifact2 = got
            .get(b"build/main.ckp".as_slice())
            .ok_or("the release has no build/main.ckp")?
            .clone();
        let manifest2 = String::from_utf8_lossy(
            got.get(b"MANIFEST".as_slice())
                .ok_or("the release has no MANIFEST")?,
        )
        .into_owned();
        let sha2 = hex(&sha256(&artifact2));
        let registry_sha = match &registry_rows[0][3] {
            Value::Text(s) => s.clone(),
            other => return fail(format!("unexpected registry value {other:?}")),
        };
        if artifact2 != artifact
            || sha2 != sha
            || sha2 != registry_sha
            || !manifest2.contains(&format!("artifact_sha256={sha2}"))
        {
            return fail("the fetched artifact does not match what was published");
        }
        t.note(
            "distrans",
            format!(
                "consumer fetched the release from the server: {}",
                describe(&fetch_stats)
            ),
        );
        t.note(
            "ghola",
            format!(
                "consumer verified every object by hash; artifact sha256 {}... matches the manifest, the SQL registry and the publisher",
                &sha2[..16]
            ),
        );

        // 8. muaddib: run only under an attenuated capability, then revoke it.
        let mut kernel = Kernel::new();
        let region = kernel.new_object(Rights::ALL);
        let runner = kernel.derive(&region, Rights::READ | Rights::EXECUTE)?;
        kernel.check(&runner, Rights::READ | Rights::EXECUTE)?;
        if kernel.check(&runner, Rights::WRITE).is_ok() {
            return fail("the runner's capability was not attenuated: it can write");
        }
        if kernel.derive(&runner, Rights::ALL).is_ok() {
            return fail("a capability was widened by derivation");
        }
        t.note(
            "muaddib",
            "granted the runner READ|EXECUTE only; WRITE is denied and widening by derive is refused",
        );

        // 9. mentat: the consumer runs it; the result must equal the publisher's.
        let consumer_result = run_artifact(&artifact2)?;
        if consumer_result != publisher_result {
            return fail(format!(
                "consumer computed {consumer_result}, publisher computed {publisher_result}"
            ));
        }
        t.note(
            "mentat",
            format!("consumer ran the fetched bytecode: main() = {consumer_result}, identical to the publisher's"),
        );

        kernel.revoke(&region)?;
        let capability_revoked = matches!(
            kernel.check(&runner, Rights::READ),
            Err(CapError::Revoked { .. })
        );
        if !capability_revoked {
            return fail("the runner's capability still works after revocation");
        }
        t.note(
            "muaddib",
            "revoked the region: the runner's capability no longer works",
        );

        Ok(Outcome {
            trace: t,
            publisher_result,
            consumer_result,
            commit,
            artifact_sha: sha,
            artifact_bytes: artifact.len(),
            push_stats,
            fetch_stats,
            consensus_crashes: cstats.crashes,
            consensus_partition_drops: cstats.dropped_by_partition,
            consensus_ticks: ticks,
            registry_rows,
            capability_revoked,
        })
    }
}
