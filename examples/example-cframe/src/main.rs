mod custom_frame;
mod custom_evm;

use custom_frame::{CustomFrame, CustomFrameFactory, FrameStats};
use custom_evm::{CustomEvm, FrameManager, demonstrate_frame_trait_objects};

use revm::{
    handler::FrameTr,
    interpreter::interpreter::EthInterpreter,
    primitives::address,
};

fn main() {
    println!("🚀 REVM Custom Frame with FrameTr Implementation\n");
    println!("This example demonstrates how to create custom frames that properly");
    println!("implement the FrameTr trait and integrate with EvmTr.\n");

    // Demonstrate manual custom frame creation with FrameTr
    println!("🔬 Custom Frame with FrameTr Example:\n");
    frame_trait_example();

    // Demonstrate frame factory usage
    println!("\n🏭 Frame Factory Example:\n");
    frame_factory_example();

    // Demonstrate custom EVM with tracking
    println!("\n⚙️ Custom EVM Integration Example:\n");
    custom_evm_example();

    // Demonstrate statistics collection
    println!("\n📊 Frame Statistics Collection:\n");
    statistics_example();
}

fn frame_trait_example() {
    println!("Creating CustomFrame that implements FrameTr...\n");

    let mut manager = FrameManager::new();

    // Create multiple frames to demonstrate the trait implementation
    let mut frame1 = manager.create_demo_frame("main_execution");
    let mut frame2 = manager.create_demo_frame("nested_call");

    println!("📦 Created frames:");
    println!("  Frame 1: {} (type: {})", frame1.tag, frame1.frame_type());
    println!("  Frame 2: {} (type: {})", frame2.tag, frame2.frame_type());

    // Simulate some execution
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Update gas usage
    frame1.update_gas_usage();
    frame2.update_gas_usage();

    // Log completion
    frame1.log_end();
    frame2.log_end();

    println!("\n✅ Frames properly implement FrameTr trait:");
    println!("  - FrameResult type: {:?}", std::any::type_name::<revm::handler::FrameResult>());
    println!("  - FrameInit type: {:?}", std::any::type_name::<revm::interpreter::interpreter_action::FrameInit>());
}

fn frame_factory_example() {
    println!("Using CustomFrameFactory for automated frame creation...\n");

    let mut factory = CustomFrameFactory::new();
    let mut frames = Vec::new();

    // Create multiple frames using the factory
    for i in 0..3 {
        use revm::{
            context_interface::journaled_state::JournalCheckpoint,
            handler::{CallFrame, CreateFrame, FrameData},
            interpreter::{FrameInput, Gas, Interpreter},
        };

        let data = if i % 2 == 0 {
            FrameData::Call(CallFrame {
                return_memory_range: 0..32,
            })
        } else {
            FrameData::Create(CreateFrame {
                created_address: address!("0000000000000000000000000000000000000042"),
            })
        };

        let mut interpreter = Interpreter::<EthInterpreter>::default();
        interpreter.gas = Gas::new(50_000 + i as u64 * 10_000);

        let frame = factory.create_frame(
            data,
            FrameInput::Empty,
            i,
            JournalCheckpoint::default(),
            interpreter,
        );

        frames.push(frame);
    }

    // Simulate execution and track frames
    for (i, frame) in frames.iter_mut().enumerate() {
        std::thread::sleep(std::time::Duration::from_millis(5 + i as u64));
        frame.update_gas_usage();
        frame.log_end();
    }

    println!("\n🎯 Factory created {} tracked frames", frames.len());
}

fn custom_evm_example() {
    println!("Integrating CustomFrame with custom EVM implementation...\n");

    // Create custom EVM
    let mut custom_evm = CustomEvm::<CustomFrame<EthInterpreter>>::new();
    let mut manager = FrameManager::new();

    println!("🔧 Created CustomEvm with FrameTr integration");

    // Create and add frames to demonstrate frame stack management
    for i in 0..3 {
        let frame = manager.create_demo_frame(&format!("evm_frame_{}", i + 1));
        println!("  Pushing frame: {}", frame.tag);
        custom_evm.push_frame(frame);
    }

    println!("  Frame stack depth: {}", custom_evm.depth());

    // Pop and process frames
    while custom_evm.depth() > 0 {
        if let Some(mut frame) = custom_evm.pop_frame() {
            frame.update_gas_usage();
            custom_evm.record_frame_stats(&frame);
            frame.log_end();
        }
    }

    println!("\n📈 EVM execution completed");
    custom_evm.print_stats();

    // Demonstrate trait objects
    println!("\n🎭 Trait Object Demonstration:");
    demonstrate_frame_trait_objects();
}

fn statistics_example() {
    println!("Collecting and analyzing frame execution statistics...\n");

    let mut stats = FrameStats::new();
    let mut manager = FrameManager::new();

    // Simulate multiple frame executions with different characteristics
    for i in 0..10 {
        let tag = format!("stats_frame_{}", i + 1);
        let mut frame = manager.create_demo_frame(&tag);

        // Simulate different execution times and gas usage
        let sleep_time = (i % 5) + 1;
        std::thread::sleep(std::time::Duration::from_millis(sleep_time as u64));

        // Simulate gas consumption
        let gas_consumed = 10_000 + (i * 5_000) as u64;
        frame.gas_used = gas_consumed;

        // Record in statistics
        stats.record_frame(&frame);

        frame.log_end();
    }

    // Print comprehensive statistics
    stats.print_stats();

    println!("\n🔍 Statistics demonstrate frame tracking capabilities:");
    println!("  - Execution time monitoring");
    println!("  - Gas consumption analysis");
    println!("  - Call depth tracking");
    println!("  - Frame type categorization");
    println!("  - Aggregate metrics collection");
}

/// Demonstrate trait object usage with FrameTr
fn demonstrate_trait_objects() {
    println!("\n🎭 Trait Object Example:\n");

    // Create a vector of trait objects
    let mut frames: Vec<Box<dyn FrameTr<FrameResult = revm::handler::FrameResult, FrameInit = revm::interpreter::interpreter_action::FrameInit>>> = Vec::new();

    let mut manager = FrameManager::new();
    let custom_frame = manager.create_demo_frame("trait_object_frame");

    // Box the frame as a trait object
    frames.push(Box::new(custom_frame));

    println!("✅ Successfully used CustomFrame as FrameTr trait object");
    println!("  Trait objects enable polymorphic frame handling");
}