use crate::{Merge, Reactive};
use paste::paste;
use std::hash::Hash;

macro_rules! merge_impl {
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

merge_impl!(0, 1,);
merge_impl!(0, 1, 2,);
merge_impl!(0, 1, 2, 3,);
merge_impl!(0, 1, 2, 3, 4,);
merge_impl!(0, 1, 2, 3, 4, 5,);
merge_impl!(0, 1, 2, 3, 4, 5, 6,);
merge_impl!(0, 1, 2, 3, 4, 5, 6, 7,);
merge_impl!(0, 1, 2, 3, 4, 5, 6, 7, 8,);
merge_impl!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9,);
merge_impl!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,);
merge_impl!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,);
