use std::collections::HashSet;
use tauri_plugin_global_shortcut::Shortcut;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    #[default]
    Interaction,
    Game,
    Free,
}

impl Mode {
    pub fn next(self) -> Self {
        match self {
            Self::Interaction => Self::Game,
            Self::Game => Self::Free,
            Self::Free => Self::Interaction,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Binding {
    pub action: String,
    pub shortcut: String,
}

pub fn defaults() -> Vec<Binding> {
    [
        ("help", "NumpadAdd"),
        ("mode", "NumpadDivide"),
        ("visibility", "Numpad0"),
        ("previous", "Numpad8"),
        ("next", "Numpad2"),
        ("pause", "Numpad5"),
        ("quieter", "Numpad4"),
        ("louder", "Numpad6"),
        ("mute", "NumpadDecimal"),
        ("corner", "Numpad7"),
        ("size", "Numpad9"),
        ("compact", "NumpadSubtract"),
        ("quit", "NumpadMultiply"),
    ]
    .into_iter()
    .map(|(action, shortcut)| Binding {
        action: action.into(),
        shortcut: shortcut.into(),
    })
    .collect()
}

pub fn validate(bindings: &[Binding]) -> Result<Vec<Shortcut>, String> {
    let expected: HashSet<_> = defaults().into_iter().map(|b| b.action).collect();
    let actual: HashSet<_> = bindings.iter().map(|b| b.action.clone()).collect();
    if bindings.len() != expected.len() || actual != expected {
        return Err("Il faut un raccourci pour chaque action.".into());
    }
    let mut used = HashSet::new();
    bindings
        .iter()
        .map(|binding| {
            let shortcut: Shortcut = binding
                .shortcut
                .parse()
                .map_err(|_| format!("Touche non reconnue : {}", binding.shortcut))?;
            if !used.insert(shortcut.id()) {
                return Err(format!(
                    "Raccourci utilisé deux fois : {}",
                    binding.shortcut
                ));
            }
            Ok(shortcut)
        })
        .collect()
}

pub fn enabled(bindings: &[Binding], mode: Mode, capturing: bool) -> Vec<Binding> {
    bindings
        .iter()
        .filter(|binding| !capturing && (mode != Mode::Free || binding.action == "mode"))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_are_complete_and_distinct() {
        assert_eq!(validate(&defaults()).unwrap().len(), 13);
    }
    #[test]
    fn all_shortcuts_stay_active_while_gaming() {
        let bindings = defaults();
        for mode in [Mode::Interaction, Mode::Game] {
            assert_eq!(enabled(&bindings, mode, false).len(), 13);
            assert!(enabled(&bindings, mode, false)
                .iter()
                .any(|b| b.action == "next"));
        }
    }
    #[test]
    fn focus_rebinding_and_capture_obey_the_same_policy() {
        let mut bindings = defaults();
        bindings
            .iter_mut()
            .find(|b| b.action == "mode")
            .unwrap()
            .shortcut = "Numpad1".into();
        assert_eq!(
            enabled(&bindings, Mode::Free, false)
                .iter()
                .find(|b| b.action == "mode")
                .unwrap()
                .shortcut,
            "Numpad1"
        );
        assert_eq!(enabled(&bindings, Mode::Free, false).len(), 1);
        for mode in [Mode::Interaction, Mode::Game, Mode::Free] {
            assert!(enabled(&bindings, mode, true).is_empty());
        }
    }
    #[test]
    fn focus_key_cycles_interaction_game_free_then_interaction() {
        let mut mode = Mode::Interaction;
        for expected in [Mode::Game, Mode::Free, Mode::Interaction] {
            mode = mode.next();
            assert_eq!(mode, expected);
        }
    }
    #[test]
    fn free_mode_reserves_only_the_activation_key() {
        let selected = enabled(&defaults(), Mode::Free, false);
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].action, "mode");
        assert_eq!(selected[0].shortcut, "NumpadDivide");
    }
    #[test]
    fn detects_equivalent_shortcuts() {
        let mut bindings = defaults();
        bindings[0].shortcut = "Ctrl+KeyA".into();
        bindings[1].shortcut = "Control+A".into();
        assert!(validate(&bindings).unwrap_err().contains("deux fois"));
    }
    #[test]
    fn rejects_incomplete_unknown_and_invalid_bindings() {
        let mut bindings = defaults();
        bindings.pop();
        assert!(validate(&bindings).is_err());
        let mut bindings = defaults();
        bindings[0].action = "shell".into();
        assert!(validate(&bindings).is_err());
        let mut bindings = defaults();
        bindings[0].shortcut = "BadKey".into();
        assert!(validate(&bindings).is_err());
    }
    #[test]
    fn accepts_custom_keys_and_modifiers() {
        let mut bindings = defaults();
        bindings[0].shortcut = "Ctrl+Shift+F10".into();
        assert!(validate(&bindings).is_ok());
        let encoded = serde_json::to_string(&bindings).unwrap();
        assert!(validate(&serde_json::from_str::<Vec<Binding>>(&encoded).unwrap()).is_ok());
    }
}
