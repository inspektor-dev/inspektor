---
title: how profiler works
description: This blog post explains how profiler works
slug: how-profiler-works
authors:
  - name: Poonai
    title: Maintainer of inspektor
    url: https://twitter.com/poonai_
    image_url: https://i.imgur.com/RNM7R6Q.jpg
tags: [rust,linux,performance]
image: /img/measure.jpg
hide_table_of_contents: false
---
![image of speedometer](/img/measure.jpg)

Many of us use profiler to measure the CPU or memory consumed by the piece of code. This led me to figure out how profilers work. 

I learn most of the things by reading the source code of the open-source projects. And, I'm grateful to all the OSS fellows who are sharing their knowledge and especially to a person like me who went to a small-town school.

If you are also interested in contributing to open source code or want to learn how to read complex project source code. I would highly recommend [Contributing to Complex Projects](https://mitchellh.com/writing/contributing-to-complex-projects) by [Mitchell Hashimoto](https://twitter.com/mitchellh).

This time to learn about the profiler, I groked a popular profiling crate pprof-rs.  This library is used to measure the CPU usage of the program. Before we start, let's just profile a sample rust program and see how pprof-rs generated profiles looks like.

Here is the modifled example program that I've taken from pprof-rs. You find the full source [here](https://gist.github.com/poonai/dab9e7e4812b65aeea82451efe81e227).


The sample program calculates the number of prime numbers from 1 to 50000.


```rust
fn main() {
    let prime_numbers = prepare_prime_numbers();

    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(100)
        .build()
        .unwrap();
    let mut v = 0;
    for i in 1..50000 {
        if i % 3 == 0 {
            if is_prime_number1(i, &prime_numbers) {
                v += 1;
            }
        } 
         else {
            if is_prime_number2(i, &prime_numbers) {
                v += 1;
            }
        }
    }
    println!("Prime numbers: {}", v);
    if let Ok(report) = guard.report().build() {
        let mut file = File::create("profile.pb").unwrap();
        let profile = report.pprof().unwrap();

        let mut content = Vec::new();
        profile.write_to_vec(&mut content).unwrap();
        file.write_all(&content).unwrap();
    };
}
```

We started profiling at the beginning of the program using `ProfilerGuardBuilder`

```rust
    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(100)
        .build()
        .unwrap();
```

At the end of the program, we generated the report and wrote the report to 
`profile.pb` file.  The generated profile report can be visualized using google's `pprof`.

```
 ~/go/bin/pprof --http=localhost:8080  profile.pb
``` 

After executing the above command, pprof will let us to visualize the profile at `http://localhost:8080`

![cpu profile of rust program](/img/primenumberprofiling.png)

From the visualized profile, you can clearly see that `is_prime_number2` have consumed more cpu than `is_prime_number1`. That's because `is_prime_number1` is used only the given number is divisible by 3.  

Now, that we learned how to profile rust program using `pprof-rs`. Let's learn how `pprof-rs` works internally. 


## Gist of cpu profilers.

Before we get into `pprof-rs` code, let's learn the cpu profilers in theory. 

In order to profile a program, `pprof-rs` will register a callback for `SIGPROF` signal, which triggerd for certain interval. Whenever the callback is invoked, current instruction pointer is passed to the callback. The instruction pointer points to the current program instruction that's been executed. From the instruction pointer we can find the respective the backtrace of the function call using `backtrace-rs`.  Each stack frame are recorded in the hashmap and respective count. 