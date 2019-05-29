use std::time::Duration;
use std::thread;
use std::cell::RefCell;

thread_local!(static NOTIFY: RefCell<bool> = RefCell::new(true));

mod task {
    use crate::NOTIFY;
    
    pub struct Task();
    
    impl Task {
        pub fn notify(&self) {
            NOTIFY.with(|f| {
                *f.borrow_mut() = true
            })         
        }
    }
    
    pub fn current() -> Task {
        Task()
    }
}

fn run<F>(mut f: F)
where
    F: Future<Item = (), Error = ()>,
{
    loop {
        if NOTIFY.with(|n| {
            if *n.borrow() {
                *n.borrow_mut() = false;
                match f.poll() {
                    Ok(Async::Ready(_)) | Err(_) => return true,
                    Ok(Async::NotReady) => (),
                }
            }
            thread::sleep(Duration::from_millis(1000));
            false
        }) { break }
    }
}





type Poll<T, E> = Result<Async<T>, E>;
trait Future {
    type Item;
    type Error;
    
 fn poll(&mut self) -> Poll<Self::Item, Self::Error>;
}

enum Async<T> {
    Ready(T),
    NotReady
}

#[derive(Default)]
struct MyFuture {
    count: u32,
  
}
struct AddOneFuture<T>(T);



impl Future for MyFuture {
    type Item = u32;
    type Error = ();
    
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        println!("Count: {}", self.count);
        println!("Fetching more .. ");
        
        match self.count {
            3 =>  Ok(Async::Ready((self.count))),
            _ => {
                self.count += 1;
                task::current().notify();
                Ok(Async::NotReady)
            }
        }
    }
}



impl<T> Future for AddOneFuture<T>
where
    T: Future,
    T::Item: std::ops::Add<u32, Output=u32>,
{
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.0.poll() {
            Ok(Async::Ready(count)) => {
                println!("");
                println!("Fetched all");
                println!("Final Count: {}", count + 1);
                Ok(Async::Ready(()))
            }
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(_) => Err(()),
        }
    }
}

fn main() {
    let my_future = MyFuture::default();
   run(AddOneFuture(my_future))
}