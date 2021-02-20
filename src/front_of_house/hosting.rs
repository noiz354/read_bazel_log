const COMPILE_GRADLE: &'static [&'static str] = &[
    r"buildconfig/appcompile/compile-customerapp.gradle",
    r"buildconfig/appcompile/compile-sellerapp.gradle",
    r"buildconfig/appcompile/compile-libraries.gradle",
];

const DEPENDENCIES_GRADLE: &'static [&'static str] = &[
    r"buildconfig/dependencies/dependency-features.gradle",
    r"buildconfig/dependencies/dependency-libraries.gradle",
];

use regex::Regex;
use std::convert::Into;
use std::fs;
use std::path::{Path, PathBuf};

struct DependencyGraph {
    project_path: String,
    email: String,
    ext: String,
}

impl DependencyGraph {
    pub fn new<T: AsRef<Path> + Clone>(project_path: T) -> DependencyGraph {
        let mut project_path = project_path.as_ref();

        let ext = find_project_ext(project_path.clone());
        for include_module in find_include_modules(project_path.clone())
            .unwrap_or(Vec::new())
            .into_iter()
        {
            let slash = include_module.replace("\':", "");
            let slash = slash.replace("\":", "");
            let slash = slash.replace("\"", "");
            let slash = slash.replace("\'", "");
            let slash = slash.replace(":", "/");

            let slash = project_path.join(slash);
            match fs::canonicalize(&Path::new(&slash)) {
                Ok(full_path) => {
                    let result = find_project_dependency2(
                        project_path,
                        &full_path,
                        "build.gradle".to_string(),
                    )
                    .unwrap_or(Vec::new());
                    println!("full_path {:?} result {:?}", full_path, result);
                }
                Err(e) => {
                    println!("{:?} not found", slash);
                    continue;
                }
            }
        }
        DependencyGraph {
            project_path: project_path.to_str().unwrap_or("invalid path").to_string(),
            email: "sekarang".to_string(),
            ext: "sekarang".to_string(),
        }
    }
}

use std::error::{self, *};
pub fn find_project_dependency2<T: AsRef<Path> + Clone>(
    project_path: T,
    full_path: T,
    extension: String,
) -> Result<Vec<String>, Box<dyn error::Error>> {
    let full_path = full_path.as_ref();
    let project_path = project_path.as_ref();

    let read_to_string = |x: PathBuf| {
        let y = if x.exists() {
            fs::read_to_string(x).unwrap_or(String::new())
        } else {
            String::new()
        };
        y
    };

    let content_full_path = if !extension.eq(&String::from("")) {
        let x = full_path.join(extension);
        read_to_string(x)
    } else {
        read_to_string(full_path.to_path_buf())
    };

    lazy_static! {
        static ref RE: Regex =
            Regex::new(r#"(?m)apply from:(?P<apply_gradle_script>\s*"(.*)")"#).unwrap();
        static ref implementation_configuration: Regex =
            Regex::new(r#"(?m)([Ii]mplementation|lintChecks)(\s+)\(?(rootProject|project\(rootProject).ext\.(\w+)\.(\w+)\)?"#).unwrap();

    }

    let mut result = Vec::new();
    for caps in RE.captures_iter(content_full_path.as_str()) {
        let apply_gradle_script = &caps["apply_gradle_script"]
            .to_string()
            .trim_matches(|c| c == '\'' || c == '\"' || c == ' ')
            .to_owned();
        let apply_gradle_script =
            apply_gradle_script.replace("$rootProject.projectDir", project_path.to_str().unwrap());

        // create absolute path for script
        let apply_gradle_script = full_path.join(apply_gradle_script);

        // simpan ini
        result.append(
            &mut find_project_dependency2(
                project_path,
                Path::new(&apply_gradle_script),
                "".to_string(),
            )
            .unwrap_or_default(),
        );

        for caps2 in implementation_configuration.captures_iter(content_full_path.as_str()) {
            // ambil 4 dan 5
            let apply_gradle_script4 = &caps2[4]
                .to_string()
                .trim_matches(|c| c == '\'' || c == '\"' || c == ' ')
                .to_owned();

            let apply_gradle_script5 = &caps2[5]
                .to_string()
                .trim_matches(|c| c == '\'' || c == '\"' || c == ' ')
                .to_owned();

            result.push(format!("{}/{}", apply_gradle_script4, apply_gradle_script5));
        }
    }

    Ok(result)
}

// this deprecated
pub fn find_project_dependency<T: AsRef<Path> + Clone>(
    project_path: T,
    full_path: T,
    extension: String,
) {
    let full_path = full_path.as_ref();
    let project_path = project_path.as_ref();
    let mut full_path2 = None;

    full_path2 = if !extension.eq(&String::from("")) {
        let x = full_path.join(extension);
        Some(x)
    } else {
        None
    };

    let content = if let Some(x) = full_path2 {
        let y = if x.exists() {
            fs::read_to_string(x).unwrap()
        } else {
            String::new()
        };

        y
    } else {
        let y = if full_path.exists() {
            fs::read_to_string(full_path).unwrap()
        } else {
            String::new()
        };

        y
    };

    // println!("content {}", content);

    lazy_static! {
        static ref RE: Regex =
            Regex::new(r#"(?m)apply from:(?P<apply_gradle_script>\s*"(.*)")"#).unwrap();
        static ref implementation_configuration: Regex =
            Regex::new(r#"(?m)([Ii]mplementation|lintChecks)(\s+)\(?(rootProject|project\(rootProject).ext\.(\w+)\.(\w+)\)?"#).unwrap();

    }

    for caps in RE.captures_iter(content.as_str()) {
        let apply_gradle_script = &caps["apply_gradle_script"]
            .to_string()
            .trim_matches(|c| c == '\'' || c == '\"' || c == ' ')
            .to_owned();
        let apply_gradle_script =
            apply_gradle_script.replace("$rootProject.projectDir", project_path.to_str().unwrap());

        // create absolute path for script
        let apply_gradle_script = full_path.join(apply_gradle_script);
        // println!("apply_gradle_script {:?}", apply_gradle_script);

        // simpan ini
        find_project_dependency(
            project_path,
            Path::new(&apply_gradle_script),
            "".to_string(),
        );

        // println!("Movie: {:?}", apply_gradle_script,);

        for caps2 in implementation_configuration.captures_iter(content.as_str()) {
            let apply_gradle_script = &caps2[4]
                .to_string()
                .trim_matches(|c| c == '\'' || c == '\"' || c == ' ')
                .to_owned();

            // ambil 4 dan 5
            let apply_gradle_script = &caps2[5]
                .to_string()
                .trim_matches(|c| c == '\'' || c == '\"' || c == ' ')
                .to_owned();

            // println!("ckckck: {:?}/{:?}", &caps2[4], apply_gradle_script,);
        }

        // result.push(
        //     caps["gradle_variable_name"]
        //         .to_string()
        //         .trim_matches(|c| c == ' ' || c == '\n')
        //         .to_owned(),
        // );
    }

    // let full_paths = DEPENDENCIES_GRADLE
    //     .iter()
    //     .map(|a| {
    //         let root_path = root_path.clone();
    //         root_path.join(a.clone())
    //     })
    //     .collect::<Vec<PathBuf>>();
}

pub fn find_project_ext<T: AsRef<Path> + Clone>(root_path: T) -> Result<Vec<String>, String> {
    let root_path = root_path.as_ref();

    lazy_static! {
        static ref RE: Regex = Regex::new(
            r#"(?m)(?P<gradle_variable_name>^\s+(\w+)\s*):(?P<gradle_path>\s"([\w:-]+)")"#
        )
        .unwrap();
    }

    let full_paths = DEPENDENCIES_GRADLE
        .iter()
        .map(|a| {
            let root_path = root_path.clone();
            root_path.join(a.clone())
        })
        .collect::<Vec<PathBuf>>();

    let mut result: Vec<String> = Vec::new();
    for full_path in full_paths {
        if !full_path.exists() {
            let x = format!(
                "full_path {} not existed",
                full_path
                    .to_str()
                    .unwrap_or("\"file path cannot be convert to str\"")
            );
            return Err(x);
        }

        let content = fs::read_to_string(full_path).unwrap();

        for caps in RE.captures_iter(content.as_str()) {
            result.push(
                caps["gradle_variable_name"]
                    .to_string()
                    .trim_matches(|c| c == ' ' || c == '\n')
                    .to_owned(),
            );
        }
    }
    return Ok(result);
}

pub fn find_include_modules<T: AsRef<Path> + Clone>(root_path: T) -> Result<Vec<String>, String> {
    let root_path = root_path.as_ref();

    lazy_static! {
        static ref RE: Regex =
            Regex::new(r#"(?m)^(?P<title>include )(?P<title2>['"]([\w:\-_]+)['"])"#).unwrap();
    }

    let full_paths = COMPILE_GRADLE
        .iter()
        .map(|a| {
            let root_path = root_path.clone();
            root_path.join(a.clone())
        })
        .collect::<Vec<PathBuf>>();

    let mut result: Vec<String> = Vec::new();
    for full_path in full_paths {
        if !full_path.exists() {
            let x = format!(
                "full_path {} not existed",
                full_path
                    .to_str()
                    .unwrap_or("\"file path cannot be convert to str\"")
            );
            return Err(x);
        }

        let content = fs::read_to_string(full_path).unwrap();

        for caps in RE.captures_iter(content.as_str()) {
            // println!(
            //     "Movie: {:?}, Released: {:?}",
            //     &caps["title"], &caps["title2"]
            // );
            result.push(caps["title2"].to_string().to_owned());
        }
    }
    return Ok(result);
}

pub fn find_all_modules_manifest<T: AsRef<Path>>(root_path: T) -> Vec<String> {
    use walkdir::WalkDir;

    WalkDir::new(root_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let mut x = None;
            if e.path()
                .to_str()
                .unwrap_or("")
                .contains("AndroidManifest.xml")
            {
                x = Some(e);
            }
            x
        })
        .map(|e| format!("{}", e.path().display()))
        .collect::<Vec<String>>()
}

pub fn add_to_waitlist() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_all_modules_manifest() {
        // this is should be
        use std::process::Command;
        let x = Command::new("pwd")
            .output()
            .expect("failed to execute process");
        let mut x = String::from_utf8(x.stdout).expect("Found invalid UTF-8");
        let len = x.len();
        x.truncate(len - 1);
        x.push_str("/example_manifest");

        assert_eq!(find_all_modules_manifest(x).len(), 3);
    }

    #[test]
    fn failed_test_find_all_modules_manifest() {
        // this is should be
        use std::process::Command;
        let x = Command::new("pwd")
            .output()
            .expect("failed to execute process");
        let mut x = String::from_utf8(x.stdout).expect("Found invalid UTF-8");
        let len = x.len();
        x.truncate(len - 1);
        x.push_str("/example_manifest/a/c");

        assert_eq!(find_all_modules_manifest(x).len(), 0);
    }

    #[test]
    fn internal() {
        // this is should be
        use std::process::Command;
        let x = Command::new("pwd")
            .output()
            .expect("failed to execute process");
        let mut x = String::from_utf8(x.stdout).expect("Found invalid UTF-8");
        let len = x.len();
        x.truncate(len - 1);

        assert_eq!(true, find_include_modules(x).is_ok());
    }

    #[test]
    fn test_find_project_ext() {
        // this is should be
        use std::process::Command;
        let x = Command::new("pwd")
            .output()
            .expect("failed to execute process");
        let mut x = String::from_utf8(x.stdout).expect("Found invalid UTF-8");
        let len = x.len();
        x.truncate(len - 1);

        assert_eq!(true, find_project_ext(x).is_ok());
    }

    #[test]
    fn err_xxx() {
        use std::process::Command;
        let x = Command::new("pwd")
            .output()
            .expect("failed to execute process");
        let mut x = String::from_utf8(x.stdout).expect("Found invalid UTF-8");
        let len = x.len();
        x.truncate(len - 1);

        // assert here
        let error_message: String =
            "full_path x/buildconfig/appcompile/compile-customerapp.gradle not existed".into();
        let result = find_include_modules("x").unwrap_err();
        assert_eq!(result, error_message);
    }

    #[test]
    fn test_dependency_graph() {
        use std::process::Command;
        let x = Command::new("pwd")
            .output()
            .expect("failed to execute process");
        let mut x = String::from_utf8(x.stdout).expect("Found invalid UTF-8");
        let len = x.len();
        x.truncate(len - 1);

        let y =
            DependencyGraph::new("/Users/normansyahputa/Documents/dev/repo/android-tokopedia-core");
    }
}
