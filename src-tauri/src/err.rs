use tauri::ipc::InvokeError;



pub struct WrapInfallible(std::convert::Infallible);
impl From<WrapInfallible> for InvokeError {
    fn from(_value: WrapInfallible) -> Self {
        unreachable!()
        //Self::from(())
    }
}

pub type SafeResult<T> = Result<T, WrapInfallible>;