mod exit;
mod run;
mod test;

pub async fn start() {
    let (run_cmd, sd) = run::RunApplication::new();
    let exit_handle = exit::ExitApplication::new(sd);
    let test_cmd = test::TestExampleBuilder::new().init_examples().build();

    wd_run::ArgsManager::new()
        .register_cmd(run::RunApplication::args(), run_cmd)
        .register_cmd(test_cmd.args(), test_cmd)
        .register_exit(exit_handle)
        .run()
        .await;
}
