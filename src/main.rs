use errors::ShellResult;
use tokio::task::JoinHandle;

mod errors;

fn spawn_input_handler() -> JoinHandle<ShellResult<()>> {
    tokio::spawn(async {
        let mut rl = rustyline::DefaultEditor::new()?;

        while let Ok(line) = rl.readline(">") {
            println!("{line}");

            if line.as_str() == "exit" {
                println!("bye, from shelly.");
                break;
            }
        }

        Ok(())
    })
}

#[tokio::main]
async fn main() -> ShellResult<()> {
    let handle = spawn_input_handler();
    handle.await?.expect("failed to read input");

    Ok(())
}
