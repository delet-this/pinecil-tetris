#![no_std]
#![no_main]

mod game;

use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::ascii::FONT_4X6;
use embedded_graphics::text::{Text, Baseline, Alignment};
use game::Tetris;
use numtoa::NumToA;

use core::cell::RefCell;

use hal::eclic::{EclicExt, Level, LevelPriorityBits};
use hal::exti::{ExtiEvent, ExtiLine, TriggerEdge};
use hal::timer::{Event, Timer};
use riscv::_export::critical_section;
use riscv::_export::critical_section::Mutex;

use panic_halt as _;

use embedded_hal::digital::v2::OutputPin;
use gd32vf103_pac as pac;
use gd32vf103xx_hal::{self as hal, prelude::*};
use hal::delay::McycleDelay;

use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};

type I2cInterfaceTypeAlias = I2CInterface<
    hal::i2c::BlockingI2c<
        pac::I2C0,
        (
            hal::gpio::gpiob::PB6<hal::gpio::Alternate<hal::gpio::OpenDrain>>,
            hal::gpio::gpiob::PB7<hal::gpio::Alternate<hal::gpio::OpenDrain>>,
        ),
    >,
>;
type DisplayTypeAlias = Ssd1306<
    I2cInterfaceTypeAlias,
    DisplaySize96x16,
    ssd1306::mode::BufferedGraphicsMode<DisplaySize96x16>,
>;

static G_DISP: Mutex<RefCell<Option<DisplayTypeAlias>>> = Mutex::new(RefCell::new(None));
static G_GAME: Mutex<RefCell<Option<Tetris>>> = Mutex::new(RefCell::new(None));
static G_TIMER1: Mutex<RefCell<Option<Timer<pac::TIMER1>>>> = Mutex::new(RefCell::new(None));
static G_DELAY: Mutex<RefCell<Option<McycleDelay>>> = Mutex::new(RefCell::new(None));

#[riscv_rt::entry]
fn main() -> ! {
    let p = pac::Peripherals::take().unwrap();

    let game = Tetris::init();

    critical_section::with(|cs| {
        G_GAME.borrow(cs).replace(Some(game));
    });

    // Use external 8MHz HXTAL and set PLL to get 96MHz system clock.
    let mut rcu = p
        .RCU
        .configure()
        .ext_hf_clock(8.mhz())
        .sysclk(96.mhz())
        .freeze();
    let mut afio = p.AFIO.constrain(&mut rcu);

    let mut delay = McycleDelay::new(&rcu.clocks);
    critical_section::with(|cs| {
        G_DELAY.borrow(cs).replace(Some(delay));
    });

    let gpioa = p.GPIOA.split(&mut rcu);
    let gpiob = p.GPIOB.split(&mut rcu);

    // left + button
    let btn_b = gpiob.pb0.into_pull_down_input();

    // right - button
    // Note that this pin is already pulled low externally via a 10K resistor
    // since it also operates the BOOT0 pin, so we don't need the internal
    // pull-down.
    let btn_a = gpiob.pb1.into_floating_input();

    pac::ECLIC::reset();
    pac::ECLIC::set_threshold_level(Level::L0);
    pac::ECLIC::set_level_priority_bits(LevelPriorityBits::L2P2);

    // timer 1 interrupt
    pac::ECLIC::setup(
        pac::Interrupt::TIMER1,
        hal::eclic::TriggerType::Level,
        hal::eclic::Level::L1,
        hal::eclic::Priority::P1,
    );
    let mut timer1 = Timer::timer1(p.TIMER1, 4.hz(), &mut rcu);
    timer1.listen(Event::Update);
    critical_section::with(|cs| {
        G_TIMER1.borrow(cs).replace(Some(timer1));
    });

    let mut exti = hal::exti::Exti::new(p.EXTI);

    // + button EXTI interrupt
    pac::ECLIC::setup(
        pac::Interrupt::EXTI_LINE1,
        hal::eclic::TriggerType::RisingEdge,
        hal::eclic::Level::L1,
        hal::eclic::Priority::P2,
    );
    afio.extiss(btn_a.port(), btn_a.pin_number());
    let extiline_a = ExtiLine::from_gpio_line(btn_a.pin_number()).unwrap();
    exti.listen(extiline_a, TriggerEdge::Rising);
    exti.gen_event(extiline_a, ExtiEvent::Enable);
    hal::exti::Exti::clear(extiline_a);

    // - button EXTI interrupt
    pac::ECLIC::setup(
        pac::Interrupt::EXTI_LINE0,
        hal::eclic::TriggerType::RisingEdge,
        hal::eclic::Level::L1,
        hal::eclic::Priority::P2,
    );
    afio.extiss(btn_b.port(), btn_b.pin_number());
    let extiline_b = ExtiLine::from_gpio_line(btn_b.pin_number()).unwrap();
    exti.listen(extiline_b, TriggerEdge::Rising);
    exti.gen_event(extiline_b, ExtiEvent::Enable);
    hal::exti::Exti::clear(extiline_b);

    unsafe {
        pac::ECLIC::unmask(pac::Interrupt::EXTI_LINE0);
        pac::ECLIC::unmask(pac::Interrupt::EXTI_LINE1);
        pac::ECLIC::unmask(pac::Interrupt::TIMER1);
        riscv::interrupt::enable();
    };

    // OLED reset: Pull low to reset.
    let mut oled_reset = gpioa
        .pa9
        .into_push_pull_output_with_state(hal::gpio::State::Low);

    let pb6_scl = gpiob.pb6.into_alternate_open_drain();
    let pb7_sda = gpiob.pb7.into_alternate_open_drain();

    // Set up i2c.
    let i2c0 = hal::i2c::BlockingI2c::i2c0(
        p.I2C0,
        (pb6_scl, pb7_sda),
        &mut afio,
        hal::i2c::Mode::Fast {
            frequency: 400_000.hz(),
            duty_cycle: hal::i2c::DutyCycle::Ratio2to1,
        },
        &mut rcu,
        1000,
        10,
        1000,
        1000,
    );

    // OLED datasheet recommends 100 ms delay on power up.
    delay.delay_ms(100);

    // Init OLED.
    oled_reset.set_high().unwrap();

    // OLED datasheet recommends 3 us delay to wait for init.
    delay.delay_us(3);

    let interface = I2CDisplayInterface::new(i2c0);
    let mut disp = Ssd1306::new(interface, DisplaySize96x16, DisplayRotation::Rotate90)
        .into_buffered_graphics_mode();
    disp.init().unwrap();

    critical_section::with(|cs| {
        G_DISP.borrow(cs).replace(Some(disp));
    });

    // disp.set_brightness(Brightness::custom(0xF1, 0x0F_u8));

    loop {
        unsafe {
            riscv::asm::wfi();
        }
    }
}

fn draw(game: &mut Tetris, disp: &mut DisplayTypeAlias) {
    disp.clear();

    let thin_stroke = PrimitiveStyle::with_stroke(BinaryColor::On, 1);

    let grid = game.get_grid();

    let character_style = MonoTextStyle::new(&FONT_4X6, BinaryColor::On);

    let mut buf = [0u8; 20];

    if game.has_ended() {
        Text::with_alignment(
            "Game",
            Point::new(8, 13),
            character_style,
            Alignment::Center,
        )
        .draw(disp).unwrap();

        Text::with_alignment(
            "over",
            Point::new(8, 23),
            character_style,
            Alignment::Center,
        )
        .draw(disp).unwrap();

        Text::with_alignment(
            "Pts",
            Point::new(8, 43),
            character_style,
            Alignment::Center,
        )
        .draw(disp).unwrap();

        Text::with_alignment(
            game.get_score().numtoa_str(10, &mut buf),
            Point::new(8, 53),
            character_style,
            Alignment::Center,
        )
        .draw(disp).unwrap();

        disp.flush().unwrap();
        return;
    }

    const VERT_OFFSET: i32 = 2*16;

    Rectangle::new(Point::new(0, 0), Size::new(1, 32))
        .into_styled(thin_stroke)
        .draw(disp)
        .unwrap();

    Rectangle::new(Point::new(0, 0), Size::new(16, 1))
        .into_styled(thin_stroke)
        .draw(disp)
        .unwrap();

    Rectangle::new(Point::new(15, 0), Size::new(1, 32))
        .into_styled(thin_stroke)
        .draw(disp)
        .unwrap();

    Rectangle::new(Point::new(0, 31), Size::new(16, 1))
        .into_styled(thin_stroke)
        .draw(disp)
        .unwrap();

    Text::with_alignment(
        game.get_score().numtoa_str(10, &mut buf),
        Point::new(8, 15),
        character_style,
        Alignment::Center,
    )
    .draw(disp).unwrap();

    if let Some(block) = game.get_block() {
        for (i, row) in block.shape.iter().enumerate() {
            for (j, bit) in row.iter().enumerate() {
                let x: i32 = (j as i32 + i32::from(block.pos.0)) * 2;
                // let y: i32 = 95 - (i as i32 + block.pos.1 as i32) * 2 - 1;
                let y: i32 = (i as i32 + i32::from(block.pos.1)) * 2 - 1 + VERT_OFFSET;
                if *bit {
                    Rectangle::new(Point::new(x, y), Size::new(2, 2))
                        .into_styled(thin_stroke)
                        .draw(disp)
                        .unwrap();
                }
            }
        }
    }

    for (i, row) in grid.iter().enumerate() {
        for (j, val) in row.iter().enumerate() {
            let x: i32 = (j as i32) * 2;
            // let y: i32 = 95 - (i as i32) * 2 - 1;
            let y: i32 = (i as i32) * 2 - 1 + VERT_OFFSET;
            if *val {
                Rectangle::new(Point::new(x, y), Size::new(2, 2))
                    .into_styled(thin_stroke)
                    .draw(disp)
                    .unwrap();
            }
        }
    }
    disp.flush().unwrap();
}

#[allow(non_snake_case)]
#[no_mangle]
fn TIMER1() {
    critical_section::with(|cs| {
        if let Some(timer1) = &mut *G_TIMER1.borrow(cs).borrow_mut() {
            timer1.clear_update_interrupt_flag();
        }

        if let Some(game) = &mut *G_GAME.borrow(cs).borrow_mut() {
            game.run();

            if let Some(disp) = &mut *G_DISP.borrow(cs).borrow_mut() {
                draw(game, disp);
            }
        }
    });
}

#[allow(non_snake_case)]
#[no_mangle]
fn EXTI_LINE0() {
    let extiline = ExtiLine::from_gpio_line(0).unwrap();

    if hal::exti::Exti::is_pending(extiline) {
        hal::exti::Exti::clear(extiline);
    }

    critical_section::with(|cs| {
        let mut ended = false;
        if let Some(game) = &mut *G_GAME.borrow(cs).borrow_mut() {
            if game.has_ended() {
                ended = true;
            }
            game.rotate_block();

            if let Some(disp) = &mut *G_DISP.borrow(cs).borrow_mut() {
                draw(game, disp);
            }
        }
        if ended {
            G_GAME.borrow(cs).replace(Some(Tetris::init()));
        }
    });
}

#[allow(non_snake_case)]
#[no_mangle]
fn EXTI_LINE1() {
    let extiline = ExtiLine::from_gpio_line(1).unwrap();
    if hal::exti::Exti::is_pending(extiline) {
        hal::exti::Exti::clear(extiline);
    }

    critical_section::with(|cs| {
        let mut ended = false;
        if let Some(game) = &mut *G_GAME.borrow(cs).borrow_mut() {
            if game.has_ended() {
                ended = true;
            }

            game.move_block();

            if let Some(disp) = &mut *G_DISP.borrow(cs).borrow_mut() {
                draw(game, disp);
            }
        }
        if ended {
            G_GAME.borrow(cs).replace(Some(Tetris::init()));
        }
    });
}
