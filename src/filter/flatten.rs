use super::{Filter, FilterBase, Internal, Tuple};
use crate::reject::CombineRejection;
use futures::future::{AndThen, MapErr, TryFutureExt};

#[derive(Clone, Copy, Debug)]
pub struct Flatten<F> {
    pub(super) filter: F,
}

impl<F, Fi, T> FilterBase for Flatten<F>
where
    F: Filter<Extract = (Fi,)>,
    Fi: Filter<Extract = T>,
    Fi::Error: CombineRejection<F::Error>,
    T: Tuple,
{
    type Extract = T;
    type Error = <Fi::Error as CombineRejection<F::Error>>::One;
    // By god, if we only had existential types available here.
    // Then we could also remove the `as fn(_) -> _`.
    type Future = AndThen<
        MapErr<
            <F as FilterBase>::Future,
            fn(
                <F as FilterBase>::Error,
            )
                -> <<Fi as FilterBase>::Error as CombineRejection<<F as FilterBase>::Error>>::One,
        >,
        MapErr<
            <Fi as FilterBase>::Future,
            fn(
                <Fi as FilterBase>::Error,
            )
                -> <<Fi as FilterBase>::Error as CombineRejection<<F as FilterBase>::Error>>::One,
        >,
        fn(
            (Fi,),
        ) -> MapErr<
            <Fi as FilterBase>::Future,
            fn(
                <Fi as FilterBase>::Error,
            )
                -> <<Fi as FilterBase>::Error as CombineRejection<<F as FilterBase>::Error>>::One,
        >,
    >;
    #[inline]
    fn filter(&self, _: Internal) -> Self::Future {
        self.filter
            .filter(Internal)
            .map_err(
                <<Fi::Error as CombineRejection<F::Error>>::One as From<F::Error>>::from
                    as fn(_) -> _,
            )
            .and_then(
                (|(inner,): (Fi,)| {
                    inner.filter(Internal).map_err(
                        <<Fi::Error as CombineRejection<F::Error>>::One as From<Fi::Error>>::from
                            as fn(_) -> _,
                    )
                }) as fn(_) -> _,
            )
    }
}
