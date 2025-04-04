use std::{
    f32::consts::PI,
    io::{stdout, Write},
};

use anyhow::Result;
use crossterm::{
    cursor::{Hide, MoveDown, MoveLeft, MoveTo, Show},
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind},
    execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor},
    terminal::{
        disable_raw_mode, enable_raw_mode, size, Clear, ClearType, DisableLineWrap, EnableLineWrap,
    },
};

const SPEED: f32 = 0.25;

pub struct LineSegment {
    pub slope: f32,
    pub intercept: f32,
    pub start: (f32, f32),
    pub end: (f32, f32),
    pub colour: Color,
}

impl LineSegment {
    pub fn from_points(start: (f32, f32), end: (f32, f32), colour: Color) -> Self {
        let slope = (end.1 - start.1) / (end.0 - start.0);
        Self {
            slope,
            intercept: -slope * start.0 + start.1,
            start,
            end,
            colour,
        }
    }

    pub fn ray(start: (f32, f32), angle: f32) -> Self {
        let slope = angle.tan();
        let end = (start.0 + 15.0 * angle.cos(), start.1 + 15.0 * angle.sin());
        Self {
            slope,
            intercept: -slope * start.0 + start.1,
            start,
            end,
            colour: Color::White,
        }
    }

    pub fn intersects(&self, other: &Self) -> Option<(f32, f32)> {
        if other.slope.is_infinite() {
            if self.slope.is_infinite() {
                if self.start.0 == other.start.0 {
                    return Some((self.start.0, 0.0)); // same line
                } else {
                    return None;
                }
            }
            if between(other.start.0, self.start.0, self.end.0)
                && between(self.find_y(other.start.0), other.start.1, other.end.1)
            {
                return Some((other.start.0, self.find_y(other.start.0)));
            }
            return None;
        }
        if self.slope.is_infinite() {
            return other.intersects(self);
        }
        let intersection = (other.intercept - self.intercept) / (self.slope - other.slope);
        return (between(intersection, self.start.0, self.end.0)
            && between(intersection, other.start.0, other.end.0))
        .then(|| (intersection, self.find_y(intersection)));
    }

    pub fn find_y(&self, x: f32) -> f32 {
        self.slope * x + self.intercept
    }
}

pub fn between(x: f32, a: f32, b: f32) -> bool {
    if a < b {
        (a..=b).contains(&x)
    } else {
        (b..=a).contains(&x)
    }
}

pub fn get_distance(point_a: (f32, f32), point_b: (f32, f32)) -> f32 {
    ((point_b.1 - point_a.1).powf(2.0) + (point_b.0 - point_a.0).powf(2.0)).sqrt()
}

fn exit_raw_mode() -> Result<()> {
    execute!(
        stdout(),
        DisableMouseCapture,
        ResetColor,
        Clear(ClearType::All),
        Show,
        EnableLineWrap
    )
    .unwrap();
    disable_raw_mode()?;
    Ok(())
}

fn render(
    size: (u16, u16),
    position: (f32, f32),
    rotation: &mut f32,
    segments: &Vec<LineSegment>,
) -> Result<()> {
    if *rotation < -PI {
        *rotation = 2.0 * PI + *rotation;
    } else if *rotation > PI {
        *rotation = *rotation - 2.0 * PI;
    }
    for y in 0..=size.1 {
        queue!(stdout(), MoveTo(0, y))?;
        if y > size.1 / 2 {
            for _ in 0..size.0 {
                queue!(stdout(), SetBackgroundColor(Color::Blue), Print(" "))?;
            }
        } else {
            for _ in 0..size.0 {
                queue!(stdout(), SetBackgroundColor(Color::Red), Print(" "))?;
            }
        }
    }
    let d_theta = 0.5 * PI / size.0 as f32;
    for x in 0..size.0 {
        let ray = LineSegment::ray(position, *rotation - (x as f32 * d_theta));
        let mut distance: Option<f32> = None;
        let mut colour = Color::White;
        for segment in segments.iter() {
            if let Some(point) = segment.intersects(&ray) {
                let new_distance = get_distance(position, point);
                if distance.is_none() || distance > Some(new_distance) {
                    distance = Some(new_distance);
                    colour = segment.colour;
                }
            }
        }
        if let Some(distance) = distance {
            let height = if distance > 5.0 {
                (size.1 as f32 * (1.0 - ((distance - 5.0) * 0.1))).round() as u16
            } else {
                size.1
            };

            let padding = (size.1 - height) / 2;
            queue!(stdout(), MoveTo(x, padding))?;
            for _ in 0..height {
                queue!(
                    stdout(),
                    SetBackgroundColor(colour),
                    Print(" "),
                    MoveDown(1),
                    MoveLeft(1),
                )?;
            }
        }
    }
    queue!(
        stdout(),
        MoveTo(0, 0),
        Print(format!(
            "x: {}, y: {}, rot: {}",
            position.0, position.1, rotation
        ))
    )?;
    stdout().flush()?;
    Ok(())
}

fn main() -> Result<()> {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |p| {
        exit_raw_mode().unwrap();
        hook(p);
    }));
    enable_raw_mode().unwrap();
    execute!(stdout(), EnableMouseCapture, Hide, DisableLineWrap).unwrap();

    let segments = vec![
        LineSegment::from_points((6.0, 1.0), (4.0, 3.0), Color::Black),
        LineSegment::from_points((4.0, 3.0), (7.0, 5.0), Color::Magenta),
        LineSegment::from_points((7.0, 5.0), (6.0, 1.0), Color::Green),
        LineSegment::from_points((2.0, 1.0), (-2.0, 1.0), Color::White),
        LineSegment::from_points((-2.0, 1.0), (-2.0, 5.0), Color::Magenta),
        LineSegment::from_points((-2.0, 5.0), (2.0, 5.0), Color::Green),
        LineSegment::from_points((2.0, 5.0), (2.0, 1.0), Color::Yellow),
    ];
    let mut position = (0.0, 0.0);
    let mut rotation = 3.0 * PI / 4.0;
    let mut last_mouse_position = None;

    loop {
        let size = size()?;
        match event::read().unwrap() {
            Event::Mouse(evt) => match evt.kind {
                MouseEventKind::Moved => {
                    render(size, position, &mut rotation, &segments)?;
                    if let Some(pos) = last_mouse_position {
                        rotation += (evt.column as i32 - pos as i32) as f32 * 0.01;
                    }
                    last_mouse_position = Some(evt.column);
                }
                _ => {}
            },
            Event::Key(key) => match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('w') => {
                    position.0 += (rotation - PI / 4.0).cos() * SPEED;
                    position.1 += (rotation - PI / 4.0).sin() * SPEED;
                    render(size, position, &mut rotation, &segments)?;
                }
                KeyCode::Char('s') => {
                    position.0 -= (rotation - PI / 4.0).cos() * SPEED;
                    position.1 -= (rotation - PI / 4.0).sin() * SPEED;
                    render(size, position, &mut rotation, &segments)?;
                }
                KeyCode::Char('a') => {
                    position.0 += (rotation + PI / 4.0).cos() * SPEED;
                    position.1 += (rotation + PI / 4.0).sin() * SPEED;
                    render(size, position, &mut rotation, &segments)?;
                }
                KeyCode::Char('d') => {
                    position.0 -= (rotation + PI / 4.0).cos() * SPEED;
                    position.1 -= (rotation + PI / 4.0).sin() * SPEED;
                    render(size, position, &mut rotation, &segments)?;
                }
                KeyCode::Char('h') => {
                    rotation += 0.05;
                    render(size, position, &mut rotation, &segments)?;
                }
                KeyCode::Char('l') => {
                    rotation -= 0.05;
                    render(size, position, &mut rotation, &segments)?;
                }
                _ => {}
            },
            _ => {}
        }
    }

    exit_raw_mode()
}
