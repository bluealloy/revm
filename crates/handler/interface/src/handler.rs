use crate::{ExecutionHandler, PostExecutionHandler, PreExecutionHandler, ValidationHandler};

pub trait Handler {
    type Validation: ValidationHandler;
    type PreExecution: PreExecutionHandler;
    type Execution: ExecutionHandler;
    type PostExecution: PostExecutionHandler;

    fn validation(&mut self) -> &mut Self::Validation;
    fn pre_execution(&mut self) -> &mut Self::PreExecution;
    fn execution(&mut self) -> &mut Self::Execution;
    fn post_execution(&mut self) -> &mut Self::PostExecution;
}
