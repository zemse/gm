use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use block2::StackBlock;
use dispatch2::{DispatchSemaphore, DispatchTime};
use objc2::runtime::Bool;
use objc2_foundation::{NSError, NSString};
use objc2_local_authentication::{LAContext, LAPolicy};

/// Prompt the user for authentication using Face ID, Touch ID, or device passcode.
/// The `msg` parameter is the reason for authentication displayed in the prompt.
/// Returns `Ok(())` if user has authenticated successfully, otherwise returns an error.
/// Errors:
/// - `AuthNotAvailable`: If neither Touch ID, Face ID, nor device passcode is set up.
/// - `AuthFailed`: If authentication failed or was cancelled by the user.
pub fn authenticate(msg: &str) -> crate::Result<()> {
    unsafe {
        let ctx = LAContext::new();

        ctx.canEvaluatePolicy_error(LAPolicy::DeviceOwnerAuthentication)
            .map_err(|_| crate::Error::AuthNotAvailable)?;

        let success = Arc::new(AtomicBool::new(false));
        let sema = Arc::new(DispatchSemaphore::new(0));
        let success_cl = Arc::clone(&success);
        let sema_cl = Arc::clone(&sema);

        let reply = StackBlock::new(move |ok: Bool, _: *mut NSError| {
            success_cl.store(ok.is_true(), Ordering::SeqCst);

            sema_cl.signal();
        })
        .copy();

        ctx.evaluatePolicy_localizedReason_reply(
            LAPolicy::DeviceOwnerAuthentication,
            &NSString::from_str(msg),
            &reply,
        );

        sema.wait(DispatchTime::FOREVER);

        if success.load(Ordering::SeqCst) {
            Ok(())
        } else {
            Err(crate::Error::AuthFailed)
        }
    }
}
