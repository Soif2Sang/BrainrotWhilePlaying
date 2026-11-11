use std::collections::HashSet;
use tauri_plugin_global_shortcut::Shortcut;

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

pub fn enabled(bindings: &[Binding], capturing: bool) -> Vec<Binding> {
    bindings.iter().filter(|_| !capturing).cloned().collect()
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
        assert_eq!(enabled(&bindings, false).len(), 13);
        assert!(enabled(&bindings, false).iter().any(|b| b.action == "next"));
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
            enabled(&bindings, false)
                .iter()
                .find(|b| b.action == "mode")
                .unwrap()
                .shortcut,
            "Numpad1"
        );
        assert!(enabled(&bindings, true).is_empty());
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
