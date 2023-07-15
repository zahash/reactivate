use crate::{Merge, Reactive};
use paste::paste;
use std::hash::Hash;

macro_rules! impl_merge {
    ( $($i:literal,)* ) => { paste!{
        impl < $([<T $i>],)* > Merge for ( $( &Reactive<[<T $i>]>, )* )
        where
            $( [<T $i>]: Clone + Default + Hash + Send + 'static, ) *
         {
            type Output = ( $([<T $i>],)* );

            fn merge(self) -> Reactive<Self::Output> {
                let values = ( $(self.$i.value(),)* );
                let combined = Reactive::new(values);

                $( self.$i.add_observer({
                    let combined = combined.clone();
                    move |val| combined.update_inplace(|c| c.$i = val.clone())
                }); )*

                combined
            }
         }
        }};
}

impl_merge!(0, 1,);
impl_merge!(0, 1, 2,);
impl_merge!(0, 1, 2, 3,);
impl_merge!(0, 1, 2, 3, 4,);
impl_merge!(0, 1, 2, 3, 4, 5,);
impl_merge!(0, 1, 2, 3, 4, 5, 6,);
impl_merge!(0, 1, 2, 3, 4, 5, 6, 7,);
impl_merge!(0, 1, 2, 3, 4, 5, 6, 7, 8,);
impl_merge!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9,);
impl_merge!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,);
impl_merge!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,);
