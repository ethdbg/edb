use crate::executive::Executive;

pub struct Client {

}

impl Client {
    pub fn new() -> Self {
    
    
    }
}



// Create one new Executive
// expose a execute/create contract method
// 'run a debug contract' (returns finalization result)
//
// calling functions like `step(), runUntil()` send events to spawned thread
//  but then running a debug contract can't return FinalizationResult right away, because it has to
//  wait for thread execution to finish
//      - fork-join?



// create two MPSC channels, one is passed into Executor



