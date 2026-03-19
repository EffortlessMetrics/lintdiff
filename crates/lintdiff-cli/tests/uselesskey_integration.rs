use std::{
    fs,
    path::{Path, PathBuf},
};

use uselesskey::{
    negative::CorruptPem, ChainSpec, Factory, RsaFactoryExt, RsaSpec, Seed, TokenFactoryExt,
    TokenSpec, X509FactoryExt,
};

#[test]
fn deterministic_runtime_crypto_fixtures_are_stable() {
    let seed =
        Seed::from_env_value(module_path!()).expect("module path is a valid deterministic seed");
    let fx = Factory::deterministic(seed);

    let rsa = fx.rsa("lintdiff-cli", RsaSpec::rs256());
    let private_pem = rsa.private_key_pkcs8_pem();
    let public_jwk = rsa.public_jwk().to_value();
    let jwks = rsa.public_jwks().to_value();
    let temp_private_key = rsa
        .write_private_key_pkcs8_pem()
        .expect("write temporary private key");

    let token = fx.token("lintdiff-cli-api", TokenSpec::api_key());
    let chain = fx.x509_chain("lintdiff-cli-tls", ChainSpec::new("lintdiff.test"));
    let bad_private_pem = rsa.private_key_pkcs8_pem_corrupt(CorruptPem::BadHeader);

    assert!(private_pem.contains("-----BEGIN PRIVATE KEY-----"));
    assert!(temp_private_key.path().exists());
    assert_eq!(public_jwk["kty"], "RSA");
    assert_eq!(public_jwk["alg"], "RS256");
    assert_eq!(jwks["keys"].as_array().map(Vec::len), Some(1));
    assert!(!token.value().is_empty());
    assert!(chain
        .leaf_cert_pem()
        .contains("-----BEGIN CERTIFICATE-----"));
    assert!(chain
        .root_cert_pem()
        .contains("-----BEGIN CERTIFICATE-----"));
    assert_eq!(
        chain
            .chain_pem()
            .matches("-----BEGIN CERTIFICATE-----")
            .count(),
        2
    );
    assert!(bad_private_pem.contains("-----BEGIN CORRUPTED KEY-----"));

    let same_rsa = fx.rsa("lintdiff-cli", RsaSpec::rs256());
    let same_token = fx.token("lintdiff-cli-api", TokenSpec::api_key());
    let same_chain = fx.x509_chain("lintdiff-cli-tls", ChainSpec::new("lintdiff.test"));

    assert_eq!(private_pem, same_rsa.private_key_pkcs8_pem());
    assert_eq!(token.value(), same_token.value());
    assert_eq!(chain.leaf_cert_pem(), same_chain.leaf_cert_pem());
}

#[test]
fn random_runtime_crypto_fixtures_are_usable() {
    let fx = Factory::random();
    let rsa = fx.rsa("lintdiff-cli-random", RsaSpec::rs256());
    let token = fx.token("lintdiff-cli-random-api", TokenSpec::bearer());
    let cert = fx.x509_self_signed(
        "lintdiff-cli-random-cert",
        uselesskey::X509Spec::self_signed("localhost"),
    );

    assert!(rsa
        .private_key_pkcs8_pem()
        .contains("-----BEGIN PRIVATE KEY-----"));
    assert_eq!(rsa.public_jwk().to_value()["kty"], "RSA");
    assert!(!token.value().is_empty());
    assert!(cert.cert_pem().contains("-----BEGIN CERTIFICATE-----"));
}

#[test]
fn repo_does_not_commit_secret_shaped_fixture_files() {
    let workspace_root = workspace_root();
    let mut flagged_paths = Vec::new();
    collect_secret_shaped_paths(&workspace_root, &mut flagged_paths).expect("scan workspace");

    assert!(
        flagged_paths.is_empty(),
        "committed secret-shaped fixtures are disallowed; generate them at runtime with uselesskey instead: {flagged_paths:#?}"
    );
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root")
        .to_path_buf()
}

fn collect_secret_shaped_paths(
    root: &Path,
    flagged_paths: &mut Vec<PathBuf>,
) -> std::io::Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            if should_skip_dir(&path) {
                continue;
            }
            collect_secret_shaped_paths(&path, flagged_paths)?;
            continue;
        }

        if is_secret_shaped_path(&path)
            || should_scan_contents(&path) && has_secret_shaped_contents(&path)?
        {
            flagged_paths.push(path);
        }
    }

    Ok(())
}

fn should_skip_dir(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some(".git" | "target")
    )
}

fn is_secret_shaped_path(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("pem" | "key" | "crt" | "cer" | "der" | "jwk" | "jwks" | "p12" | "pfx")
    )
}

fn should_scan_contents(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str() == "fixtures")
}

fn has_secret_shaped_contents(path: &Path) -> std::io::Result<bool> {
    let Ok(raw) = fs::read(path) else {
        return Ok(false);
    };

    if raw.contains(&0) {
        return Ok(false);
    }

    let text = String::from_utf8_lossy(&raw);

    Ok([
        "-----BEGIN PRIVATE KEY-----",
        "-----BEGIN RSA PRIVATE KEY-----",
        "-----BEGIN EC PRIVATE KEY-----",
        "-----BEGIN CERTIFICATE-----",
        "\"keys\": [",
        "\"kty\": \"RSA\"",
        "\"kty\": \"EC\"",
        "\"kty\": \"OKP\"",
        "\"kty\": \"oct\"",
    ]
    .iter()
    .any(|needle| text.contains(needle)))
}
