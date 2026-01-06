use clap::Parser as ClapParser;
use clap_derive::{Parser, Subcommand};
use jupyter::*;
use lang::{Environment, Evaluator, Lexer, Parser};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

struct Context {
    sockets: JupyterKernelSockets,
    environment: SharedEnvironment,
}

// Here be dragons! Proceed with caution.
struct SharedEnvironment(Rc<RefCell<Environment>>);
unsafe impl Send for SharedEnvironment {}
unsafe impl Sync for SharedEnvironment {}

#[async_trait]
impl JupyterKernelProtocol for Context {
    fn language_info(&self) -> LanguageInfo {
        LanguageInfo::new("santa-lang", "santa-lang")
            .with_file_extensions(".santa", "text/santa")
            .with_version(env!("CARGO_PKG_VERSION"))
    }

    async fn running(&mut self, code: ExecutionRequest) -> ExecutionReply {
        let lexer = Lexer::new(&code.code);
        let mut parser = Parser::new(lexer);

        let program = match parser.parse() {
            Ok(parsed) => parsed,
            Err(error) => {
                self.sockets.send_executed(error.message).await;
                return ExecutionReply::new(false, code.execution_count);
            }
        };

        let result = {
            let environment = self.environment.0.clone();
            let evaluator = Evaluator::new().evaluate_with_environment(&program, environment);
            match evaluator {
                Ok(result) => result.to_string(),
                Err(error) => error.message,
            }
        };

        self.sockets.send_executed(result).await;

        ExecutionReply::new(true, code.execution_count)
    }

    async fn bind_execution_socket(&self, sender: UnboundedSender<ExecutionResult>) {
        self.sockets.bind_execution_socket(sender).await
    }
}

#[derive(Parser)]
pub struct Application {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Start(Box<StartAction>),
    Install(Box<InstallAction>),
    Uninstall(Box<UninstallAction>),
}

impl Application {
    pub fn run(&self) -> JupyterResult<()> {
        let config = Context {
            sockets: JupyterKernelSockets::default(),
            environment: SharedEnvironment(Environment::new()),
        };

        match &self.command {
            Command::Start(v) => v.run(config),
            Command::Install(v) => v.run(config),
            Command::Uninstall(v) => v.run(config),
        }
    }
}

fn main() -> JupyterResult<()> {
    tracing_subscriber::fmt::init();
    Application::parse().run()
}
