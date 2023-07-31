use crate::{Merge, Reactive};
use paste::paste;

macro_rules! impl_merge_for_tuple {
    ( $($i:literal,)* ) => { paste!{
        impl < $([<T $i>],)* > Merge for ( $( &Reactive<[<T $i>]>, )* )
        where
            $( [<T $i>]: Clone + Default + Send + 'static, ) *
         {
            type Output = ( $([<T $i>],)* );

            fn merge(self) -> Reactive<Self::Output> {
                let values = ( $(self.$i.value(),)* );
                let combined = Reactive::new(values);

                $( self.$i.add_observer({
                    let combined = combined.clone();
                    // we know for sure that the value inside 'combined' did change
                    // because 'combined' stores the reactive values as-is without any transformation
                    // eg: (&Reactive<String>, &Reactive<usize>, ...) -> Reactive<(String, usize, ...)>
                    // so if the parent reactive changes, the 'combined' will definitely change.
                    // Therefore 'unchecked' is fine.
                    move |val| combined.update_inplace_unchecked(|c| c.$i = val.clone())
                }); )*

                combined
            }
         }
        }};
}

impl_merge_for_tuple!(0, 1,);
impl_merge_for_tuple!(0, 1, 2,);
impl_merge_for_tuple!(0, 1, 2, 3,);
impl_merge_for_tuple!(0, 1, 2, 3, 4,);
impl_merge_for_tuple!(0, 1, 2, 3, 4, 5,);
impl_merge_for_tuple!(0, 1, 2, 3, 4, 5, 6,);
impl_merge_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7,);
impl_merge_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8,);
impl_merge_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9,);
impl_merge_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,);
impl_merge_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,);
impl_merge_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,);
impl_merge_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13,);
impl_merge_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14,);
impl_merge_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,);
