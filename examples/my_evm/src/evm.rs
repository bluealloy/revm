use revm::{
    context::{ContextError, ContextSetters, ContextTr, CreateScheme, Evm, FrameStack},
    handler::{
        evm::FrameTr, instructions::EthInstructions, EthFrame, EthPrecompiles, EvmTr,
        FrameInitOrResult, FrameResult, ItemOrResult,
    },
    inspector::{InspectorEvmTr, JournalExt},
    interpreter::{
        interpreter::EthInterpreter, interpreter_action::FrameInit, CreateInputs, FrameInput,
    },
    primitives::{Address, Bytes, U256},
    Database, Inspector,
};

/// MyEvm variant of the EVM.
///
/// This struct demonstrates how to create a custom EVM implementation by wrapping
/// the standard REVM components. It combines a context (CTX), an inspector (INSP),
/// and the standard Ethereum instructions, precompiles, and frame execution logic.
///
/// The generic parameters allow for flexibility in the underlying database and
/// inspection capabilities while maintaining the standard Ethereum execution semantics.
#[derive(Debug)]
pub struct MyEvm<CTX: ContextTr, INSP> {
    /// Inner EVM type.
    pub evm: Evm<
        CTX,
        INSP,
        EthInstructions<EthInterpreter, CTX>,
        EthPrecompiles,
        EthFrame<EthInterpreter>,
    >,
    /// Handler for Frame init, it allows calling subcalls.
    pub call_handler: FrameFn<CTX, INSP>,
}

/// Handler for Frame init, it allows calling subcalls.
pub type FrameFn<CTX, INSP> =
    fn(
        &mut MyEvm<CTX, INSP>,
        frame_input: &mut <EthFrame<EthInterpreter> as FrameTr>::FrameInit,
    )
        -> Result<Option<FrameResult>, ContextError<<<CTX as ContextTr>::Db as Database>::Error>>;

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
            evm: Evm {
                ctx,
                inspector,
                instruction: EthInstructions::new_mainnet(),
                precompiles: EthPrecompiles::default(),
                frame_stack: FrameStack::new(),
            },
            call_handler: |evm, frame_input| {
                // check if it is call to specific address.
                if let FrameInput::Call(call_inputs) = &frame_input.frame_input {
                    // only continue if call is zero
                    if call_inputs.target_address != Address::ZERO {
                        return Ok(None);
                    }
                }
                // create a subcall context
                let sub_call = FrameInit {
                    depth: frame_input.depth + 1,
                    memory: frame_input.memory.new_child_context(),
                    frame_input: FrameInput::Create(Box::new(CreateInputs {
                        caller: Address::ZERO,
                        scheme: CreateScheme::Create,
                        value: U256::ZERO,
                        init_code: Bytes::new(),
                        gas_limit: 0,
                    })),
                };
                // call subcall in recursion.
                let result = evm.run_exec_loop(sub_call)?;

                // propagate the subcall result to the upper call.
                Ok(Some(result))
            },
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
    type Frame = EthFrame<EthInterpreter>;

    #[inline]
    fn all(
        &self,
    ) -> (
        &Self::Context,
        &Self::Instructions,
        &Self::Precompiles,
        &FrameStack<Self::Frame>,
    ) {
        self.evm.all()
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
        self.evm.all_mut()
    }

    #[inline]
    fn frame_init(
        &mut self,
        mut frame_input: <Self::Frame as FrameTr>::FrameInit,
    ) -> Result<
        ItemOrResult<&mut Self::Frame, <Self::Frame as FrameTr>::FrameResult>,
        ContextError<<<Self::Context as ContextTr>::Db as Database>::Error>,
    > {
        let call_handle = self.call_handler;
        if let Some(result) = (call_handle)(self, &mut frame_input)? {
            return Ok(ItemOrResult::Result(result));
        }

        self.evm.frame_init(frame_input)
    }

    #[inline]
    fn frame_run(
        &mut self,
    ) -> Result<
        FrameInitOrResult<Self::Frame>,
        ContextError<<<Self::Context as ContextTr>::Db as Database>::Error>,
    > {
        self.evm.frame_run()
    }

    #[inline]
    fn frame_return_result(
        &mut self,
        frame_result: <Self::Frame as FrameTr>::FrameResult,
    ) -> Result<
        Option<<Self::Frame as FrameTr>::FrameResult>,
        ContextError<<<Self::Context as ContextTr>::Db as Database>::Error>,
    > {
        self.evm.frame_return_result(frame_result)
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
        self.evm.all_inspector()
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
        self.evm.all_mut_inspector()
    }
}
