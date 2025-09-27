use revm::{
    context::{ContextSetters, ContextTr, FrameStack},
    handler::{
        evm::{ContextDbError, FrameInitResult, FrameTr},
        instructions::{EthInstructions, InstructionProvider},
        EthFrame, EthPrecompiles, EvmTr, FrameInitOrResult,
    },
    inspector::{InspectorEvmTr, JournalExt},
    interpreter::interpreter::EthInterpreter,
    Inspector,
};

use crate::frame::MyFrame;

/// MyEvm variant of the EVM.
///
/// This struct demonstrates how to create a custom EVM implementation by wrapping
/// the standard REVM components. It combines a context (CTX), an inspector (INSP),
/// and the standard Ethereum instructions, precompiles, and frame execution logic.
///
/// The generic parameters allow for flexibility in the underlying database and
/// inspection capabilities while maintaining the standard Ethereum execution semantics.
#[derive(Debug)]
pub struct MyEvm<CTX, INSP> {
    /// [`revm::context_interface::ContextTr`] of the EVM, it is used to fetch data from database.
    pub ctx: CTX,
    /// Inspector of the EVM it is used to inspect the EVM.
    /// Its trait are defined in revm-inspector crate.
    pub inspector: INSP,
    /// Instructions provider of the EVM it is used to execute instructions.
    /// `InstructionProvider` trait is defined in revm-handler crate.
    pub instruction: EthInstructions<EthInterpreter, CTX>,
    /// Precompile provider of the EVM it is used to execute precompiles.
    /// `PrecompileProvider` trait is defined in revm-handler crate.
    pub precompiles: EthPrecompiles,
    /// Frame that is going to be executed.
    pub frame_stack: FrameStack<MyFrame<EthInterpreter>>,
}

impl<CTX: ContextTr, INSP> MyEvm<CTX, INSP> {
    /// Creates a new instance of MyEvm with the provided context and inspector.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The execution context that manages state, environment, and journaling
    /// * `inspector` - The inspector for debugging and tracing execution
    ///
    /// # Returns
    ///
    /// A new MyEvm instance configured with:
    /// - The provided context and inspector
    /// - Mainnet instruction set
    /// - Default Ethereum precompiles
    /// - A fresh frame stack for execution
    pub fn new(ctx: CTX, inspector: INSP) -> Self {
        Self {
            ctx,
            inspector,
            instruction: EthInstructions::new_mainnet(),
            precompiles: EthPrecompiles::default(),
            frame_stack: FrameStack::new(),
        }
    }
}

impl<CTX: ContextTr, INSP> EvmTr for MyEvm<CTX, INSP>
where
    CTX: ContextTr,
{
    type Context = CTX;
    type Instructions = EthInstructions<EthInterpreter, CTX>;
    type Precompiles = EthPrecompiles;
    type Frame = MyFrame<EthInterpreter>;

    fn all(
        &self,
    ) -> (
        &Self::Context,
        &Self::Instructions,
        &Self::Precompiles,
        &FrameStack<Self::Frame>,
    ) {
        (
            &self.ctx,
            &self.instruction,
            &self.precompiles,
            &self.frame_stack,
        )
    }

    fn all_mut(
        &mut self,
    ) -> (
        &mut Self::Context,
        &mut Self::Instructions,
        &mut Self::Precompiles,
        &mut FrameStack<Self::Frame>,
    ) {
        (
            &mut self.ctx,
            &mut self.instruction,
            &mut self.precompiles,
            &mut self.frame_stack,
        )
    }

    /// Initializes the frame for the given frame input. Frame is pushed to the frame stack.
    #[inline]
    fn frame_init(
        &mut self,
        frame_input: <Self::Frame as FrameTr>::FrameInit,
    ) -> Result<FrameInitResult<'_, Self::Frame>, ContextDbError<CTX>> {
        let is_first_init = self.frame_stack.index().is_none();
        let mut new_frame = if is_first_init {
            self.frame_stack.start_init()
        } else {
            self.frame_stack.get_next()
        };
        let frame = new_frame.get(|| MyFrame {
            eth_frame: EthFrame::invalid(),
        });

        let ctx = &mut self.ctx;
        let precompiles = &mut self.precompiles;
        let res = frame.eth_frame.init(ctx, precompiles, frame_input)?;
        let token = new_frame.consume();

        Ok(res.map_frame(|_| {
            if is_first_init {
                unsafe { self.frame_stack.end_init(token) };
            } else {
                unsafe { self.frame_stack.push(token) };
            }
            self.frame_stack.get_mut()
        }))
    }

    /// Run the frame from the top of the stack. Returns the frame init or result.
    #[inline]
    fn frame_run(&mut self) -> Result<FrameInitOrResult<Self::Frame>, ContextDbError<CTX>> {
        let (context, instructions, _, frame_stack) = self.all_mut();
        let frame = frame_stack.get_mut();

        Ok(frame
            .eth_frame
            .run_and_process_next_action(context, instructions.instruction_table()))
    }

    /// Returns the result of the frame to the caller. Frame is popped from the frame stack.
    #[inline]
    fn frame_return_result(
        &mut self,
        result: <Self::Frame as FrameTr>::FrameResult,
    ) -> Result<Option<<Self::Frame as FrameTr>::FrameResult>, ContextDbError<Self::Context>> {
        if self.frame_stack.get().is_finished() {
            self.frame_stack.pop();
        }
        if self.frame_stack.index().is_none() {
            return Ok(Some(result));
        }
        self.frame_stack
            .get_mut()
            .eth_frame
            .return_result::<_, ContextDbError<Self::Context>>(&mut self.ctx, result)?;
        Ok(None)
    }
}

impl<CTX: ContextTr, INSP> InspectorEvmTr for MyEvm<CTX, INSP>
where
    CTX: ContextSetters<Journal: JournalExt>,
    INSP: Inspector<CTX, EthInterpreter>,
{
    type Inspector = INSP;

    fn all_inspector(
        &self,
    ) -> (
        &Self::Context,
        &FrameStack<Self::Frame>,
        &Self::Instructions,
        &Self::Precompiles,
        &Self::Inspector,
    ) {
        (
            &self.ctx,
            &self.frame_stack,
            &self.instruction,
            &self.precompiles,
            &self.inspector,
        )
    }

    fn all_mut_inspector(
        &mut self,
    ) -> (
        &mut Self::Context,
        &mut FrameStack<Self::Frame>,
        &mut Self::Instructions,
        &mut Self::Precompiles,
        &mut Self::Inspector,
    ) {
        (
            &mut self.ctx,
            &mut self.frame_stack,
            &mut self.instruction,
            &mut self.precompiles,
            &mut self.inspector,
        )
    }
}
