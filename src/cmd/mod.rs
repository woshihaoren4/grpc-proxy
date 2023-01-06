mod run;

pub async fn start(){
    let run_cmd = run::RunApplication::new();

    wd_run::ArgsManager::new()
        .register_cmd(run::RunApplication::args(), run_cmd)
        .run()
        .await;
}