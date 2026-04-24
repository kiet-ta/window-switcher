use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct HyprctlActiveWindow {
    address: String,
    #[serde(alias = "monitorID")]
    monitor: i32,
}

#[test]
fn test_active() {
    let output = std::process::Command::new("hyprctl")
        .args(&["activewindow", "-j"])
        .output()
        .unwrap();
    let json_str = String::from_utf8_lossy(&output.stdout);
    let res: Result<HyprctlActiveWindow, _> = serde_json::from_str(&json_str);
    assert!(res.is_ok(), "{}", res.unwrap_err());
}
