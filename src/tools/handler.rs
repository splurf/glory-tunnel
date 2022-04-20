use std::{
    io::Result,
    thread::{sleep, spawn, JoinHandle},
    time::Duration,
};

/**
 * Continuously call the given instructions every millisecond to reduce cpu load
 */
pub fn create_handler<T: FnMut(&mut bool) -> Result<()> + 'static + Send>(
    mut f: T,
) -> JoinHandle<Result<()>> {
    spawn(move || -> Result<()> {
        const TIMER: Duration = Duration::from_millis(1);

        //  Determinate for when the handler needs to stop calling
        let mut cont = true;

        loop {
            f(&mut cont)?;

            if !cont {
                break Ok(());
            } else {
                sleep(TIMER)
            }
        }
    })
}
