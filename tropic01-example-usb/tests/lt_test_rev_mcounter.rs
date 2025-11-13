/// Functional test for monotonic counter API
///
/// This test mirrors the C SDK lt_test_rev_mcounter.c functional test.
/// It verifies mcounter_init, mcounter_get, and mcounter_update operations.
///
/// **IMPORTANT**: This test is REVERSIBLE - it initializes all counters to 0 in
/// cleanup. However, it requires hardware access via USB transport.
///
/// Run with: cargo test --test lt_test_rev_mcounter -- --nocapture
/// --test-threads=1
use dummy_pin::DummyPin;
use rand_core::OsRng;
use rand_core::RngCore;
use tropic01::MCOUNTER_VALUE_MAX;
use tropic01::MCounterIndex;
use tropic01::Tropic01;
use tropic01::X25519Dalek;
use tropic01::keys::SH0PRIV_PROD;
use tropic01::keys::SH0PUB_PROD;
use tropic01_example_usb::port::UsbDevice;
use x25519_dalek::PublicKey;
use x25519_dalek::StaticSecret;

const PORT_NAME: &str = "/dev/ttyACM0";
const BAUD_RATE: u32 = 115200;
const MAX_DECREMENTS: u32 = 100;

/// Get all monotonic counter indices
fn all_mcounter_indices() -> [MCounterIndex; 16] {
    [
        MCounterIndex::Index0,
        MCounterIndex::Index1,
        MCounterIndex::Index2,
        MCounterIndex::Index3,
        MCounterIndex::Index4,
        MCounterIndex::Index5,
        MCounterIndex::Index6,
        MCounterIndex::Index7,
        MCounterIndex::Index8,
        MCounterIndex::Index9,
        MCounterIndex::Index10,
        MCounterIndex::Index11,
        MCounterIndex::Index12,
        MCounterIndex::Index13,
        MCounterIndex::Index14,
        MCounterIndex::Index15,
    ]
}

/// Setup: Create device and start secure session
fn setup_device() -> Result<Tropic01<UsbDevice, DummyPin>, Box<dyn std::error::Error>> {
    let usb_device = UsbDevice::new(PORT_NAME, BAUD_RATE)?;
    let mut tropic = Tropic01::new(usb_device);

    let ehpriv = StaticSecret::random_from_rng(OsRng);
    let ehpub = PublicKey::from(&ehpriv);
    let shpub = SH0PUB_PROD.into();
    let shpriv = SH0PRIV_PROD.into();

    println!("Starting secure session with slot 0...");
    tropic.session_start(&X25519Dalek, shpub, shpriv, ehpub, ehpriv, 0)?;
    println!("Secure session established.");

    Ok(tropic)
}

/// Cleanup: reset all counters to 0
fn cleanup_counters(
    tropic: &mut Tropic01<UsbDevice, DummyPin>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- Cleanup: Resetting all counters to 0 ---");

    for (idx, index) in all_mcounter_indices().iter().enumerate() {
        println!("Initializing monotonic counter {} to zero...", idx);
        tropic.mcounter_init(*index, 0)?;
    }

    println!("Cleanup complete.");
    Ok(())
}

#[test]
#[ignore] // Requires hardware - run with: cargo test -- --ignored --nocapture
fn test_mcounter_operations() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("==============================================");
    println!("lt_test_rev_mcounter - Rust Implementation");
    println!("==============================================\n");

    let mut tropic = setup_device()?;
    let mut rng = OsRng;

    // Cleanup on panic
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("\nTest panicked: {}", panic_info);
        eprintln!("Attempting cleanup...");
        if let Ok(mut device) = setup_device() {
            let _ = cleanup_counters(&mut device);
        }
    }));

    println!("--- Test: Random Initialization and Decrements ---");
    for (idx, index) in all_mcounter_indices().iter().enumerate() {
        // Generate random init value (within valid range)
        let mut buf = [0u8; 4];
        rng.fill_bytes(&mut buf);
        let init_val = u32::from_le_bytes(buf) & MCOUNTER_VALUE_MAX;

        println!("\n[Counter {}] Initializing with value: {}", idx, init_val);
        tropic.mcounter_init(*index, init_val)?;

        // Verify initialization
        let val = tropic.mcounter_get(*index)?;
        assert_eq!(
            val, init_val,
            "Counter {} init failed: expected {}, got {}",
            idx, init_val, val
        );
        println!("[Counter {}] Verified initialization: {}", idx, val);

        // Re-initialize with same value (should succeed but not change value)
        println!(
            "[Counter {}] Re-initializing with same value (should be idempotent)...",
            idx
        );
        tropic.mcounter_init(*index, init_val)?;
        let val_after_reinit = tropic.mcounter_get(*index)?;
        assert_eq!(
            val_after_reinit, init_val,
            "Counter {} changed after re-init",
            idx
        );

        // Try a few decrements (up to MAX_DECREMENTS or until we hit 0)
        println!("[Counter {}] Testing decrements from {}...", idx, init_val);
        let mut expected_val = init_val;
        let mut decrement_count = 0;

        while expected_val > 0 && decrement_count < MAX_DECREMENTS {
            // Read and verify current value
            let current = tropic.mcounter_get(*index)?;
            assert_eq!(
                current, expected_val,
                "Counter {} mismatch: expected {}, got {}",
                idx, expected_val, current
            );

            // Decrement
            tropic.mcounter_update(*index)?;
            expected_val -= 1;
            decrement_count += 1;
        }

        let final_val = tropic.mcounter_get(*index)?;
        println!(
            "[Counter {}] After {} decrements: {}",
            idx, decrement_count, final_val
        );
    }

    println!("\n--- Test: Decrement to Zero ---");
    for (idx, index) in all_mcounter_indices().iter().enumerate() {
        // Small random value (0-99) for faster test
        let mut buf = [0u8; 4];
        rng.fill_bytes(&mut buf);
        let small_init_val: u32 = u32::from_le_bytes(buf) % 100;

        println!(
            "\n[Counter {}] Initializing with small value: {}",
            idx, small_init_val
        );
        tropic.mcounter_init(*index, small_init_val)?;

        // Decrement all the way to 0
        let mut expected_val = small_init_val;
        while expected_val > 0 {
            let current = tropic.mcounter_get(*index)?;
            assert_eq!(
                current, expected_val,
                "Counter {} mismatch before decrement: expected {}, got {}",
                idx, expected_val, current
            );

            tropic.mcounter_update(*index)?;
            expected_val -= 1;
        }

        // Verify we're at 0
        let final_val = tropic.mcounter_get(*index)?;
        assert_eq!(
            final_val, 0,
            "Counter {} should be 0, got {}",
            idx, final_val
        );
        println!("[Counter {}] Successfully decremented to 0", idx);

        // Try to decrement when at 0 (should fail)
        println!(
            "[Counter {}] Attempting to decrement below 0 (should fail)...",
            idx
        );
        match tropic.mcounter_update(*index) {
            Ok(_) => panic!("Counter {} allowed decrement below 0!", idx),
            Err(e) => println!("[Counter {}] Correctly failed: {:?}", idx, e),
        }
    }

    println!("\n--- All tests passed! ---");

    // Cleanup
    cleanup_counters(&mut tropic)?;

    Ok(())
}

#[test]
#[ignore] // Requires hardware
fn test_mcounter_invalid_init_value() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("==============================================");
    println!("Test: Invalid Init Value");
    println!("==============================================\n");

    let mut tropic = setup_device()?;

    // Try to init with value > MCOUNTER_VALUE_MAX (should fail)
    println!("Attempting to init counter with value > MCOUNTER_VALUE_MAX...");
    let invalid_val = MCOUNTER_VALUE_MAX + 1;

    match tropic.mcounter_init(MCounterIndex::Index0, invalid_val) {
        Ok(_) => panic!("Should have rejected value > MCOUNTER_VALUE_MAX"),
        Err(e) => {
            println!("Correctly rejected invalid value: {:?}", e);
            assert!(matches!(e, tropic01::Error::InvalidParameter));
        },
    }

    println!("Test passed!");
    Ok(())
}

#[test]
#[ignore] // Requires hardware
fn test_mcounter_boundary_values() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("==============================================");
    println!("Test: Boundary Values");
    println!("==============================================\n");

    let mut tropic = setup_device()?;

    // Test min value (0)
    println!("Testing minimum value (0)...");
    tropic.mcounter_init(MCounterIndex::Index0, 0)?;
    let val = tropic.mcounter_get(MCounterIndex::Index0)?;
    assert_eq!(val, 0);
    println!("✓ Min value (0) OK");

    // Test max value (MCOUNTER_VALUE_MAX)
    println!("Testing maximum value (0xFFFFFFFE)...");
    tropic.mcounter_init(MCounterIndex::Index1, MCOUNTER_VALUE_MAX)?;
    let val = tropic.mcounter_get(MCounterIndex::Index1)?;
    assert_eq!(val, MCOUNTER_VALUE_MAX);
    println!("✓ Max value (0xFFFFFFFE) OK");

    // Decrement from max
    println!("Decrementing from max...");
    tropic.mcounter_update(MCounterIndex::Index1)?;
    let val = tropic.mcounter_get(MCounterIndex::Index1)?;
    assert_eq!(val, MCOUNTER_VALUE_MAX - 1);
    println!("✓ Decrement from max OK: {}", val);

    // Cleanup
    cleanup_counters(&mut tropic)?;

    println!("Test passed!");
    Ok(())
}
