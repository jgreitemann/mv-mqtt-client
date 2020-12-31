#[macro_export]
macro_rules! weak {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = std::rc::Rc::downgrade($n); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = std::rc::Rc::downgrade($n); )+
            move |$(weak!(@param $p),)+| $body
        }
    );
    (&$($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = std::rc::Rc::downgrade(&$n); )+
            move |$(weak!(@param $p),)+| $body
        }
    );
}
