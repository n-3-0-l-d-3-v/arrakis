use std::fmt;

/// One thing one layer did, with the evidence for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub layer: &'static str,
    pub what: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Trace {
    pub steps: Vec<Step>,
}

impl Trace {
    pub fn note(&mut self, layer: &'static str, what: impl Into<String>) {
        self.steps.push(Step {
            layer,
            what: what.into(),
        });
    }

    /// The distinct layers that took part, in order of first appearance.
    pub fn layers(&self) -> Vec<&'static str> {
        let mut out: Vec<&'static str> = Vec::new();
        for s in &self.steps {
            if !out.contains(&s.layer) {
                out.push(s.layer);
            }
        }
        out
    }
}

impl fmt::Display for Trace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, s) in self.steps.iter().enumerate() {
            writeln!(f, "{:>2}. [{:<9}] {}", i + 1, s.layer, s.what)?;
        }
        Ok(())
    }
}
