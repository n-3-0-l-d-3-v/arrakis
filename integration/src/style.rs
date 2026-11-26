//! The terminal look (docs/design, direction A: amber retro terminal).
//!
//! Two independent switches:
//! - `color`: ANSI colour. Colour carries meaning only: labels are dim, layer
//!   names amber, success green, refusals red. Foreground only, so the
//!   user's own terminal background is kept.
//! - `unicode`: `─` rules and `▌` phase bars instead of ASCII. Only for a real
//!   terminal: Windows PowerShell 5.1 decodes redirected output with the OEM
//!   code page, which turns UTF-8 box characters into garbage in logs.
//!
//! `Style::PLAIN` (neither) reproduces the original ASCII output exactly.

/// What a piece of text is, which decides its colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// The `ARRAKIS` title block (reverse video).
    Title,
    /// A stage name: PUBLISH, REPLICATE, CONSUME.
    Phase,
    /// Field labels and explanatory text.
    Label,
    /// A subsystem name.
    Layer,
    /// Success: `ok`, `VERIFIED`.
    Good,
    /// A refusal, mismatch or failure.
    Bad,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Style {
    pub color: bool,
    pub unicode: bool,
}

impl Style {
    /// ASCII, no colour: what a log file or a pipe receives.
    pub const PLAIN: Style = Style {
        color: false,
        unicode: false,
    };
    /// Colour and box-drawing characters: an interactive terminal.
    pub const FANCY: Style = Style {
        color: true,
        unicode: true,
    };

    /// `text` wrapped in the colour for `role`, or unchanged without colour.
    pub fn paint(&self, role: Role, text: &str) -> String {
        if !self.color {
            return text.to_string();
        }
        let sgr = match role {
            Role::Title => "1;38;5;16;48;5;179",
            Role::Phase => "1;38;5;214",
            Role::Label => "38;5;137",
            Role::Layer => "38;5;179",
            Role::Good => "1;38;5;107",
            Role::Bad => "1;38;5;203",
        };
        format!("\x1b[{sgr}m{text}\x1b[0m")
    }

    /// A full-width horizontal rule.
    pub fn rule(&self) -> String {
        let c = if self.unicode { "─" } else { "-" };
        c.repeat(74)
    }

    /// The marker in front of a stage name (empty in ASCII, as before).
    pub fn bar(&self) -> String {
        if self.unicode {
            self.paint(Role::Phase, "▌")
        } else {
            String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_is_ascii_without_escape_codes() {
        let s = Style::PLAIN;
        for role in [
            Role::Title,
            Role::Phase,
            Role::Label,
            Role::Layer,
            Role::Good,
            Role::Bad,
        ] {
            assert_eq!(s.paint(role, "x"), "x");
        }
        let all = format!("{}{}", s.rule(), s.bar());
        assert!(all.is_ascii() && !all.contains('\x1b'));
    }

    #[test]
    fn fancy_colours_and_resets() {
        let p = Style::FANCY.paint(Role::Bad, "REFUSED");
        assert!(p.starts_with("\x1b[") && p.ends_with("\x1b[0m") && p.contains("REFUSED"));
        assert!(Style::FANCY.rule().starts_with('─'));
    }
}
