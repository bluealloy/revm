use crate::frame::MyFrame;
use revm::{
    context::{ContextSetters, ContextTr, FrameStack, OutFrame},
    handler::{
        evm::{ContextDbError, FrameInitResult, FrameTr},
        instructions::{EthInstructions, InstructionProvider},
        EthFrame, EthPrecompiles, EvmTr, FrameInitOrResult,
    },
    inspector::{InspectorEvmTr, JournalExt},
    interpreter::interpreter::EthInterpreter,
    primitives::hardfork::SpecId,
    Inspector,
};

/// MyEvm variant of the EVM.
///
/// Implements [`EvmTr`] manually as the stock implementation is only provided
/// for `Evm` parameterized with [`EthFrame`]. Frame handling is delegated to
/// the [`EthFrame`] wrapped inside [`MyFrame`].
#[derive(Debug)]
pub struct MyEvm<CTX, INSP> {
    /// [`ContextTr`] of the EVM, it is used to fetch data from database.
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
    /// The custom frame stack that is going to be executed.
    pub frame_stack: FrameStack<MyFrame>,
}

impl<CTX: ContextTr, INSP> MyEvm<CTX, INSP> {
    /// Creates a new instance of MyEvm with the provided context and inspector.
    pub fn new(ctx: CTX, inspector: INSP) -> Self {
        Self {
            ctx,
            inspector,
            instruction: EthInstructions::new_mainnet_with_spec(SpecId::default()),
            precompiles: EthPrecompiles::new(SpecId::default()),
            frame_stack: FrameStack::new(),
        }
    }
}

impl<CTX: ContextTr, INSP> EvmTr for MyEvm<CTX, INSP> {
    type Context = CTX;
    type Instructions = EthInstructions<EthInterpreter, CTX>;
    type Precompiles = EthPrecompiles;
    type Frame = MyFrame;

    #[inline]
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

    #[inline]
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

        // Materialize the custom frame and initialize the wrapped EthFrame in place.
        // `invalid()` must stay allocation-free as an early result overwrites it without drop.
        let frame = new_frame.get(MyFrame::invalid);
        let res = EthFrame::init_with_context(
            OutFrame::new_init(&mut frame.eth_frame),
            &mut self.ctx,
            &mut self.precompiles,
            frame_input,
        )?;
        let token = new_frame.consume();

        Ok(res.map_item(|_inner_token| {
            if is_first_init {
                unsafe { self.frame_stack.end_init(token) };
            } else {
                unsafe { self.frame_stack.push(token) };
            }
            self.frame_stack.get()
        }))
    }

    /// Run the frame from the top of the stack. Returns the frame init or result.
    ///
    /// If frame has returned result it would mark it as finished.
    #[inline]
    fn frame_run(&mut self) -> Result<FrameInitOrResult<Self::Frame>, ContextDbError<CTX>> {
        let frame = self.frame_stack.get();
        let context = &mut self.ctx;
        let instructions = &mut self.instruction;

        let action = frame.eth_frame.interpreter.run_plain(
            instructions.instruction_table(),
            instructions.gas_table(),
            context,
        );

        frame
            .eth_frame
            .process_next_action(context, action)
            .inspect(|i| {
                if i.is_result() {
                    frame.eth_frame.set_finished(true);
                }
            })
    }

    /// Returns the result of the frame to the caller. Frame is popped from the frame stack.
    /// Consumes the frame result or returns it if there is more frames to run.
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
            .get()
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
        &Self::Instructions,
        &Self::Precompiles,
        &FrameStack<Self::Frame>,
        &Self::Inspector,
    ) {
        (
            &self.ctx,
            &self.instruction,
            &self.precompiles,
            &self.frame_stack,
            &self.inspector,
        )
    }

    fn all_mut_inspector(
        &mut self,
    ) -> (
        &mut Self::Context,
        &mut Self::Instructions,
        &mut Self::Precompiles,
        &mut FrameStack<Self::Frame>,
        &mut Self::Inspector,
    ) {
        (
            &mut self.ctx,
            &mut self.instruction,
            &mut self.precompiles,
            &mut self.frame_stack,
            &mut self.inspector,
        )
    }
}
