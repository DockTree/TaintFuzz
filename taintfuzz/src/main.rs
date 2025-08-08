include!(concat!(env!("OUT_DIR"), "/frida-bindings.rs"));



fn main()  -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        frida_init();
        let manager = frida_device_manager_new();
        if manager.is_null() {
            return Err("Failed to create device manager".into());
        }
        frida_deinit();
    }
    Ok(())
}

// // Taint tracking structures
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct TaintInfo {
//     pub source_offset: usize,
//     pub size: usize,
//     pub generation: u64,
// }

// pub struct TaintTracker {
//     // Maps memory addresses to taint information
//     tainted_memory: Arc<Mutex<HashMap<u64, TaintInfo>>>,
//     // Track which file descriptors are tainted (stdin = 0)
//     tainted_fds: Arc<Mutex<HashSet<i32>>>,
//     // Current generation for tracking taint flow
//     current_generation: Arc<Mutex<u64>>,
//     // Track interesting taint flows
//     taint_flows: Arc<Mutex<Vec<TaintFlow>>>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct TaintFlow {
//     pub from_addr: u64,
//     pub to_addr: u64,
//     pub size: usize,
//     pub instruction_addr: u64,
//     pub generation: u64,
// }

// impl TaintTracker {
//     pub fn new() -> Self {
//         let mut tainted_fds = HashSet::new();
//         tainted_fds.insert(0); // stdin is always considered tainted
        
//         Self {
//             tainted_memory: Arc::new(Mutex::new(HashMap::new())),
//             tainted_fds: Arc::new(Mutex::new(tainted_fds)),
//             current_generation: Arc::new(Mutex::new(0)),
//             taint_flows: Arc::new(Mutex::new(Vec::new())),
//         }
//     }

//     pub fn mark_memory_tainted(&self, addr: u64, size: usize, source_offset: usize) {
//         let mut memory = self.tainted_memory.lock().unwrap();
//         let mut generation = self.current_generation.lock().unwrap();
//         *generation += 1;
        
//         for i in 0..size {
//             memory.insert(addr + i as u64, TaintInfo {
//                 source_offset: source_offset + i,
//                 size: 1,
//                 generation: *generation,
//             });
//         }
        
//         println!("🏷️  Marked memory 0x{:x}..0x{:x} as tainted (gen: {})", 
//                  addr, addr + size as u64, *generation);
//     }

//     pub fn is_memory_tainted(&self, addr: u64) -> Option<TaintInfo> {
//         let memory = self.tainted_memory.lock().unwrap();
//         memory.get(&addr).cloned()
//     }

//     pub fn propagate_taint(&self, src_addr: u64, dst_addr: u64, size: usize, ip: u64) {
//         let memory = self.tainted_memory.lock().unwrap();
//         let mut flows = self.taint_flows.lock().unwrap();
        
//         // Check if source is tainted
//         if let Some(taint_info) = memory.get(&src_addr) {
//             // Record the taint flow
//             flows.push(TaintFlow {
//                 from_addr: src_addr,
//                 to_addr: dst_addr,
//                 size,
//                 instruction_addr: ip,
//                 generation: taint_info.generation,
//             });
            
//             drop(memory); // Release read lock
//             drop(flows);
            
//             // Propagate taint to destination
//             self.mark_memory_tainted(dst_addr, size, taint_info.source_offset);
            
//             println!("🔄 Taint propagated: 0x{:x} -> 0x{:x} (size: {}, IP: 0x{:x})", 
//                      src_addr, dst_addr, size, ip);
//         }
//     }

//     pub fn get_taint_flows(&self) -> Vec<TaintFlow> {
//         self.taint_flows.lock().unwrap().clone()
//     }

//     pub fn clear_taint(&self) {
//         self.tainted_memory.lock().unwrap().clear();
//         self.taint_flows.lock().unwrap().clear();
//         *self.current_generation.lock().unwrap() = 0;
//     }
// }

// // Observer that tracks taint information
// #[derive(Serialize, Deserialize, SerdeAny)]
// pub struct TaintObserver {
//     name: String,
//     taint_tracker: Arc<TaintTracker>,
// }

// impl TaintObserver {
//     pub fn new(name: &str, taint_tracker: Arc<TaintTracker>) -> Self {
//         Self {
//             name: name.to_string(),
//             taint_tracker,
//         }
//     }

//     pub fn get_taint_flows(&self) -> Vec<TaintFlow> {
//         self.taint_tracker.get_taint_flows()
//     }
// }

// impl Named for TaintObserver {
//     fn name(&self) -> &str {
//         &self.name
//     }
// }

// impl<S> Observer<S> for TaintObserver
// where
//     S: State,
// {
//     fn pre_exec(&mut self, _state: &mut S, _input: &S::Input) -> Result<(), Error> {
//         // Clear previous taint information before each execution
//         self.taint_tracker.clear_taint();
//         Ok(())
//     }

//     fn post_exec(&mut self, _state: &mut S, _input: &S::Input, _exit_kind: &ExitKind) -> Result<(), Error> {
//         let flows = self.taint_tracker.get_taint_flows();
//         println!("📊 Execution completed with {} taint flows", flows.len());
//         Ok(())
//     }
// }

// // Feedback that uses taint information to guide fuzzing
// #[derive(Serialize, Deserialize, SerdeAny)]
// pub struct TaintFeedback {
//     name: String,
//     interesting_flows: HashSet<(u64, u64)>, // (from_addr, to_addr) pairs we've seen
// }

// impl TaintFeedback {
//     pub fn new(name: &str) -> Self {
//         Self {
//             name: name.to_string(),
//             interesting_flows: HashSet::new(),
//         }
//     }
// }

// impl Named for TaintFeedback {
//     fn name(&self) -> &str {
//         &self.name
//     }
// }

// impl<S> Feedback<S> for TaintFeedback
// where
//     S: State,
// {
//     fn is_interesting<EM, OT>(&mut self, _state: &mut S, _manager: &mut EM, _input: &S::Input, observers: &OT, _exit_kind: &ExitKind) -> Result<bool, Error>
//     where
//         EM: libafl::events::EventManager<S>,
//         OT: ObserversTuple<S>,
//     {
//         // Get the taint observer
//         if let Some(taint_observer) = observers.match_name::<TaintObserver>("taint") {
//             let flows = taint_observer.get_taint_flows();
            
//             for flow in &flows {
//                 let flow_pair = (flow.from_addr, flow.to_addr);
//                 if !self.interesting_flows.contains(&flow_pair) {
//                     self.interesting_flows.insert(flow_pair);
//                     println!("✨ New taint flow discovered: 0x{:x} -> 0x{:x}", 
//                              flow.from_addr, flow.to_addr);
//                     return Ok(true);
//                 }
//             }
//         }
        
//         Ok(false)
//     }
// }

// // Frida script that hooks read() syscall and memory operations
// const FRIDA_SCRIPT: &str = r#"
// // Global taint tracker reference (will be set from Rust)
// var taintTracker = null;

// // Hook read() syscall
// Interceptor.attach(Module.getExportByName(null, "read"), {
//     onEnter: function(args) {
//         this.fd = args[0].toInt32();
//         this.buf = args[1];
//         this.count = args[2].toInt32();
        
//         console.log("[FRIDA] read() called: fd=" + this.fd + ", buf=0x" + 
//                    this.buf.toString(16) + ", count=" + this.count);
//     },
//     onLeave: function(retval) {
//         var bytesRead = retval.toInt32();
//         if (bytesRead > 0 && this.fd === 0) { // stdin
//             console.log("[FRIDA] Marking " + bytesRead + " bytes as tainted at 0x" + 
//                        this.buf.toString(16));
            
//             // Call back to Rust taint tracker
//             send({
//                 type: "taint_memory",
//                 addr: this.buf.toUInt64(),
//                 size: bytesRead,
//                 source_offset: 0
//             });
//         }
//     }
// });

// // Hook common memory operations for taint propagation
// var memcpyPtr = Module.getExportByName(null, "memcpy");
// if (memcpyPtr) {
//     Interceptor.attach(memcpyPtr, {
//         onEnter: function(args) {
//             var dst = args[0];
//             var src = args[1];
//             var size = args[2].toInt32();
            
//             send({
//                 type: "taint_propagate",
//                 src_addr: src.toUInt64(),
//                 dst_addr: dst.toUInt64(),
//                 size: size,
//                 ip: this.context.pc.toUInt64()
//             });
//         }
//     });
// }

// // Hook strcpy for string taint propagation
// var strcpyPtr = Module.getExportByName(null, "strcpy");
// if (strcpyPtr) {
//     Interceptor.attach(strcpyPtr, {
//         onEnter: function(args) {
//             var dst = args[0];
//             var src = args[1];
//             var srcStr = src.readCString();
//             var size = srcStr ? srcStr.length + 1 : 0;
            
//             if (size > 0) {
//                 send({
//                     type: "taint_propagate",
//                     src_addr: src.toUInt64(),
//                     dst_addr: dst.toUInt64(),
//                     size: size,
//                     ip: this.context.pc.toUInt64()
//                 });
//             }
//         }
//     });
// }

// console.log("[FRIDA] Taint tracking hooks installed");
// "#;

// // Custom Frida executor with taint tracking
// pub struct TaintedFridaExecutor<H, OT, S>
// where
//     H: FnMut(&BytesInput) -> ExitKind,
//     OT: ObserversTuple<S>,
//     S: State,
// {
//     inner: FridaInProcessExecutor<H, OT, S>,
//     taint_tracker: Arc<TaintTracker>,
// }

// impl<H, OT, S> TaintedFridaExecutor<H, OT, S>
// where
//     H: FnMut(&BytesInput) -> ExitKind,
//     OT: ObserversTuple<S>,
//     S: State,
// {
//     pub fn new(
//         harness: H,
//         observers: OT,
//         taint_tracker: Arc<TaintTracker>,
//     ) -> Result<Self, Error> {
//         let gum = Gum::obtain();
//         let mut frida_options = FridaOptions::default();
//         frida_options.enable_asan = false; // Disable for performance
        
//         let inner = FridaInProcessExecutor::new(
//             &gum,
//             harness,
//             observers,
//             &frida_options,
//         )?;

//         Ok(Self {
//             inner,
//             taint_tracker,
//         })
//     }

//     pub fn install_hooks(&mut self) -> Result<(), Error> {
//         // Install the Frida script
//         let script = self.inner.gum_mut().create_script(FRIDA_SCRIPT)?;
        
//         // Set up message handler for communication with Frida script
//         let taint_tracker = Arc::clone(&self.taint_tracker);
//         script.set_message_handler(move |message, _data| {
//             if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&message) {
//                 if let Some(msg_type) = msg["type"].as_str() {
//                     match msg_type {
//                         "taint_memory" => {
//                             if let (Some(addr), Some(size), Some(offset)) = (
//                                 msg["addr"].as_u64(),
//                                 msg["size"].as_u64(),
//                                 msg["source_offset"].as_u64(),
//                             ) {
//                                 taint_tracker.mark_memory_tainted(addr, size as usize, offset as usize);
//                             }
//                         }
//                         "taint_propagate" => {
//                             if let (Some(src), Some(dst), Some(size), Some(ip)) = (
//                                 msg["src_addr"].as_u64(),
//                                 msg["dst_addr"].as_u64(),
//                                 msg["size"].as_u64(),
//                                 msg["ip"].as_u64(),
//                             ) {
//                                 taint_tracker.propagate_taint(src, dst, size as usize, ip);
//                             }
//                         }
//                         _ => {}
//                     }
//                 }
//             }
//         });
        
//         script.load()?;
//         Ok(())
//     }
// }

// impl<H, OT, S> Executor<BytesInput, S> for TaintedFridaExecutor<H, OT, S>
// where
//     H: FnMut(&BytesInput) -> ExitKind,
//     OT: ObserversTuple<S>,
//     S: State,
// {
//     fn run_target(&mut self, state: &mut S, input: &BytesInput) -> Result<ExitKind, Error> {
//         self.inner.run_target(state, input)
//     }
// }

// // Example usage function
// pub fn create_taint_fuzzer() -> Result<(), Error> {
//     println!("🚀 Starting LibAFL Frida Taint Analyzer Demo");
    
//     // Create taint tracker
//     let taint_tracker = Arc::new(TaintTracker::new());
    
//     // Create observers
//     let taint_observer = TaintObserver::new("taint", Arc::clone(&taint_tracker));
//     let observers = tuple_list!(taint_observer);
    
//     // Create a simple harness (replace with your target)
//     let harness = |input: &BytesInput| {
//         // Simulate feeding input to stdin of target program
//         // In a real scenario, this would execute your target binary
//         println!("📥 Running harness with {} bytes of input", input.bytes().len());
        
        
        
//         ExitKind::Ok
//     };
    
//     // Create tainted executor
//     let mut executor = TaintedFridaExecutor::new(
//         harness,
//         observers,
//         Arc::clone(&taint_tracker),
//     )?;
    
//     // Install taint tracking hooks
//     executor.install_hooks()?;
    
//     println!("✅ Taint analyzer setup complete!");
//     println!("🎯 Target binary should now be instrumented for taint tracking");
//     println!("📋 Hooks installed for: read(), memcpy(), strcpy()");
    
//     Ok(())
// }

// fn main() -> Result<(), Error> {
//     create_taint_fuzzer()
// }

// // Additional helper functions for advanced taint analysis
// impl TaintTracker {
//     pub fn analyze_taint_coverage(&self, input_size: usize) -> f64 {
//         let memory = self.tainted_memory.lock().unwrap();
//         let tainted_bytes = memory.len();
        
//         if input_size > 0 {
//             (tainted_bytes as f64) / (input_size as f64)
//         } else {
//             0.0
//         }
//     }
    
//     pub fn get_critical_taint_flows(&self) -> Vec<TaintFlow> {
//         let flows = self.taint_flows.lock().unwrap();
        
//         // Filter for flows that might indicate critical operations
//         flows.iter()
//             .filter(|flow| {
//                 // Example: flows involving larger data movements might be more interesting
//                 flow.size > 8 || 
//                 // Or flows that cross certain address boundaries
//                 (flow.from_addr >> 20) != (flow.to_addr >> 20)
//             })
//             .cloned()
//             .collect()
//     }
// }