use glob::glob;

pub fn match_pattern(patterns: &Vec<String>) -> Result<Vec<String>, String> {
    let nso_run = match std::env::var("NSO_RUN_DIR") {
        Ok(x) => x,
        Err(_) => return Err("Expected environment variable: NSO_RUN_DIR".to_string()),
    };

    let log_files = glob(&format!("{}/logs/ncs-python-vm-*", nso_run))
        .unwrap()
        .map(|x| x.unwrap())
        .collect::<Vec<_>>();

    if log_files.len() == 0 {
        return Err(format!("Couldn't find any log files in {}/logs/", nso_run));
    }

    let matches_patterns = |filename: &String| -> bool {
        for pattern in patterns {
            if !filename.contains(pattern) {
                return false;
            }
        }

        true
    };

    let matches: Vec<String> = log_files
        .iter()
        .map(|path| path.file_name().unwrap().to_str().unwrap().to_string())
        .filter(matches_patterns)
        .collect();

    return Ok(matches);
}
