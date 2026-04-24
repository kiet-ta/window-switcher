use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct HyprctlWorkspace {
    id: i32,
}

#[derive(Deserialize, Debug)]
struct HyprctlClient {
    address: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    class: String,
    mapped: bool,
    #[serde(default)]
    monitor: Option<i32>,
    #[serde(default)]
    workspace: Option<HyprctlWorkspace>,
}

fn main() {
    let output = std::process::Command::new("hyprctl")
        .args(&["clients", "-j"])
        .output()
        .unwrap();
    let res: Result<Vec<HyprctlClient>, _> = serde_json::from_slice(&output.stdout);
    println!("{:#?}", res);
}
