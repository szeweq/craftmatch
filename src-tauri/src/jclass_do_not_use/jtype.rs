
pub trait Type {}
pub trait MemberType {}

pub enum OfField {}
pub enum OfMethod {}
pub enum OfClass {}

impl Type for OfField {}
impl Type for OfMethod {}
impl Type for OfClass {}

impl MemberType for OfField {}
impl MemberType for OfMethod {}