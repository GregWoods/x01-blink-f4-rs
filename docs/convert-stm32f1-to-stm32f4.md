# Converting a Simple HAL-based STM32F1 blinky to STM32F4

![STM32F411 Black Pill]({{ site.url }}{{ site.baseurl }}/assets/images/blackpill.jpg)

Yes, I know I could just grab the blink example from the stm32f4xx-hal repo. But this much more time-consuming way teaches more about both chips, and increses
my familiarity with the all-important reference manuals.

* Clone the F1 repo, give ```clone``` a second parameter to change the name of the folder you clone into
* Then disconnect it from the original remote, because you wouldn't want to push the converted code to the original repo

```shell
git clone https://github.com/GregWoods/stm32f1-01-blink.git stm32f4-01-blink
git remote remove origin
```

And disconnect from the remote. We wouldn't want to accidentally push our 'F4' code out to the 'F1' repo

```shell
git remote remove origin
```

## Change the Setup

* ```.cargo/config```
  * change ```thumbv7m-none-eabi``` to ```thumbv7em-none-eabihf``` (2 instances)
  * change ```arm-none-eabi-gdb``` to ```arm-none-eabihf-gdb``` (1 instance)
* ```cargo.toml```
  * change your package name from ```stm32f1-01-blink``` to ```stm32f4-01-blink```
  * change the hal dependency to ```[dependencies.stm32f1xx-hal]```
  * and change the features list: remove ```stm32f103```, use ```stm32f411``` (or ```stm32f401`` for the cheaper verison of the black pill)
  * Add a version number to ```stm32f4xx-hal```. Currently 0.7.0 (we need this the first time, becaue we don't have a valid value in cargo.lock) Trying to build will report a list of possible versions)
  * remove the ```medium`` feature from the hal crate
* ```memory.x```
  * The FLASH and RAM start addresses are the same
  * But the sizes for the stm32f411 are: FLASH 512K; RAM: 128K  (as far as I can tell)
* ```.vscode/launch.json```
  * replace ```STM32F103VCT6``` with ```STM32F411CEU6```
  * Replace ```target/stm32f1x.cfg``` with ```target/stm32f4x.cfg```
  * Change the svd from ```STM32F103.svd``` to ```STM32F411.svd```
  * Change the executable path, replace ```thumbv7m-none-eabi``` with ```thumbv7em-none-eabihf```
* Go and download [STM32F411.svd](https://www.st.com/en/microcontrollers-microprocessors/stm32f411.html#resource)
* ```main.rs```
  * change the use statement from ```stm32f1xx_hal``` to ```stm32f4xxx_hal```
  * replace ```pac``` from the 'use', with ```stm32``` - an odd naming inconsistency between the stm32f1 and stm32f4 hal projects
  * ```rustup target install thumbv7em-none-eabihf``` to install the missing bits of the toolchain

## Change the Code

Run ```cargo build``` and fix the bugs one by one referring to the hal repo: [github.com/stm32-rs/stm32f4xx-hal](https://github.com/stm32-rs/stm32f4xx-hal)

I'm not entirely sure you'll get the errors come back in the same order as listed here! It is likely I fixed more than one at once.

## Error 1

```Rust
16 | use stm32f1xx_hal::{
   |     ^^^^^^^^^^^^^ use of undeclared type or module `stm32f1xx_hal`
```

An easy one. Change ```stm32f1xx_hal``` to ```stm32f4xx_hal```

## Error 2

```Rust
18 |     pac,
   |     ^^^ no `pac` in the root
```

In this hal, the peripheral access crate (pac) is called stm32.
Which leads to our next fix...

## Error 3

```Rust
    |     let dp = pac::Peripherals::take().unwrap();
    |              ^^^ use of undeclared type or module `pac`
```

Change to

```Rust
    let dp = stm32::Peripherals::take().unwrap();
````

## Error 4

```Rust
    |     let mut flash = dp.FLASH.constrain();
    |                              ^^^^^^^^^ method not found in `stm32f4::stm32f411::FLASH`
```

'constrain' is not a thing in the Reference Manual. So this is something added by the HAL, probably to keep the rust compiler happy. In the hal code, inside flash.rs is the following comment...
    /// Constrains the FLASH peripheral to play nicely with the other abstractions

The stm32f4xx-hal doesn't have a flash.rs. A quick search in the github repo shows it uses stm32.FLASH from the PAC.
Before I went down a rabbit hole, I noticed that the probably fix for the next will mean we don't need to use FLASH at this time.
So **I remove the line**. I will have to revisit this sometime, but not today!

## Error 5

```Rust
    |     let clocks = rcc.cfgr.freeze(&mut flash.acr);
    |                           ^^^^^^ expected 0 parameters
```

'freeze' doesn't appear to be a feature of RCC_CFGR registers as far as I can tell. This is something "Rusty".
According to 'freeze' in the stm32f1xx-hal, an 'acr' parameter exists, but is not used!
It looks like the stm32f4 hal has simply removed this unused param, so we should be able to remove it here. Easy!

## Error 6

```Rust
    |     let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
    |                              ^^^^^ expected 0 parameters
```

The syntax of 'splitting' the GPIO port remains a mystery to me for now, and passing rcc.apb2 is even more confusing. The fact that the stm32f4 doesn't require this parameter is all I need to know for now!

## Error 7

```Rust
     |
     |     let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
     |                              ^^^^^^^^^^^^^^^^^^^^^ expected 0 parameters
```

Examine the Reference Manual and HAL.
In the **stm32f4** reference manual, there are no search results for CRL or CRH. Things are obviously done a bit differently to the **stm32f1**, where ```CRH`` is 'Port Configuration Register High'

### stm32f1

For port configuration, the MODE bits(2 bits per gpio line) and CNF bits (also 2 bits per 'pin') for each GPIO pin are interleaved, so we can only store 8 pins worth of setting within 32 bits.

![images/stm32f1-port-config-crh.png]({{ site.url }}{{ site.baseurl }}/assets/images/stm32f1-port-config-crh.png)

The CRL register is the same, but for GPIO pins 0..7

### stm32f4

The arragement is simpler for this chip. The settings for all 16 pins on a single GPIO port are stored in one 32 bit register.

![images/stm32f4-port-mode-register.png]({{ site.url }}{{ site.baseurl }}/assets/images/stm32f4-port-mode-register.png)

Therefore the HAL simply doesn't need to refer to CRH or CRL, it just uses the MODE register directly.

## Error 8

```Rust
    |     let mut timer = Timer::syst(cp.SYST, &clocks).start_count_down(1.hz());
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected 3 parameters
```

Looking at the ```stm32f4xx-hal``` code, in ```timer.rs```, we find:

```Rust
    impl Timer<SYST> {
        /// Configures the SYST clock as a periodic count down timer
        pub fn syst<T>(mut syst: SYST, timeout: T, clocks: Clocks) -> Self
        where
            T: Into<Hertz>,
```

well, this is handy - the timeout in Hertz is set in the constructor, instead of in the chained ```start_count_down```.

Fixing that has shown another error

```Rust
    |
    |     let mut timer = Timer::syst(cp.SYST, 1.hz(), &clocks);
    |                                                  ^^^^^^^
    |                                                  |
    |                                                  expected struct `stm32f4xx_hal::rcc::Clocks`, found `&stm32f4xx_hal::rcc::Clocks`
    |                                                  help: consider removing the borrow: `clocks`
```

As is often the case, the compiler suggests the fix. I'm unsure of the consequences of giving syst ownership of clocks, but that is for another day.

Our converted line looks like:

```Rust
    let mut timer = Timer::syst(cp.SYST, 1.hz(), clocks);
```

We can fix a couple of compiler warnings about variables which don't need to be mutable, and everything is good.

This should all now build, and the debugger can step into the code.