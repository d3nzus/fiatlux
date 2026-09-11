use super::*;

#[test]
fn tokenizes_simple_ident() {
    let tokens = tokenize("main.js");
    assert_eq!(tokens, vec![Token::Ident("main.js".to_string())]);
}

#[test]
fn strips_comments() {
    let tokens = tokenize("# a comment\nmain.js");
    assert_eq!(tokens, vec![Token::Ident("main.js".to_string())]);
}

#[test]
fn parses_nested_directories() {
    let bp = parse_source("directories { src { main.js } }").unwrap();
    match &bp.directories[0] {
        FsNode::Dir(name, children) => {
            assert_eq!(name, "src");
            assert_eq!(children.len(), 1);
            assert!(matches!(&children[0], FsNode::File(f) if f == "main.js"));
        }
        _ => panic!("expected Dir"),
    }
}

#[test]
fn parses_scripts_without_trailing_comma() {
    let bp = parse_source(r#"scripts { "npm install", "npm test" }"#).unwrap();
    assert_eq!(bp.scripts, vec!["npm install", "npm test"]);
}

#[test]
fn rejects_malformed_input() {
    let result = parse_source("directories { src {");
    assert!(result.is_err());
}

#[test]
fn generate_fs_creates_real_files() {
    let tmp = tempfile::tempdir().unwrap();
    let bp = parse_source("directories { src { main.js } README.md }").unwrap();
    generate_fs(&bp.directories, tmp.path()).unwrap();

    assert!(tmp.path().join("src/main.js").exists());
    assert!(tmp.path().join("README.md").exists());
}

#[test]
fn extensionless_file_gets_txt_appended() {
    let tmp = tempfile::tempdir().unwrap();
    let bp = parse_source("directories { README }").unwrap();
    generate_fs(&bp.directories, tmp.path()).unwrap();

    assert!(tmp.path().join("README.txt").exists());
}