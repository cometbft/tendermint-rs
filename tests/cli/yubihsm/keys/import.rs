//! Integration tests for the `yubihsm keys import` subcommand

use cli;

#[test]
fn keys_import_command_test() {
    #[allow(unused_mut)]
    let mut args = vec!["yubihsm", "keys", "import"];

    #[cfg(feature = "yubihsm-mock")]
    args.extend_from_slice(&["-c", super::KMS_CONFIG_PATH]);
    args.extend_from_slice(&["-p", super::PRIV_VALIDATOR_CONFIG_PATH]);
    // key_id:
    args.extend_from_slice(&["1"]);

    let out = cli::run_successfully(args.as_slice());

    assert_eq!(true, out.status.success());
    assert_eq!(true, out.stderr.is_empty());
    assert_eq!(
        true,
        String::from_utf8(out.stdout)
            .unwrap()
            .trim()
            .starts_with("Imported key #1:")
    );
}
