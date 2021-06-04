extern crate gtk;
extern crate gio;    

use gtk::prelude::*;
use gio::prelude::*;
use chrono::prelude::*;
use chrono::{Duration, FixedOffset, TimeZone};
use core::f64::consts::PI;


use gtk::{Builder,Window, Button, SpinButton, Calendar };

use std::env::args;


/**
 @brief days from 1 january
  
 @parm day this day of month
 @parm month this month
 @parm year this year
 @return the days from 1 jan
*/

fn hdate_get_day_of_year  (day:i32, month:i32, year:i32) -> i32 {
    
    /* get today's julian day number */
    let mut jd:i32 = (1461 * (year + 4800 + (month - 14) / 12)) / 4 +
	    (367 * (month - 2 - 12 * ((month - 14) / 12))) / 12 -
	    (3 * ((year + 4900 + (month - 14) / 12) / 100)) / 4 + day;
    
    /* subtract the julian day of 1/1/year and add one */
    jd = jd - ((1461 * (year + 4799)) / 4 +
	    367 * 11 / 12 - (3 * ((year + 4899) / 100)) / 4);

    return jd
}


/**
 @brief utc sun times for altitude at a gregorian date

 Returns the sunset and sunrise times in minutes from 00:00 (utc time)
 if sun altitude in sunrise is deg degries.
 This function only works for altitudes sun realy is.
 If the sun never get to this altitude, the returned sunset and sunrise values 
 will be negative. This can happen in low altitude when latitude is 
 nearing the pols in winter times, the sun never goes very high in 
 the sky there.

 @param day this day of month
 @param month this month
 @param year this year
 @param longitude longitude to use in calculations
 @param latitude latitude to use in calculations
 @param deg degrees of sun's altitude (0 -  Zenith .. 90 - Horizon)
 @param sunrise return the utc sunrise in minutes
 @param sunset return the utc sunset in minutes
*/

fn hdate_get_utc_sun_time_deg (day:i32, month:i32, year:i32, latitude:f64, longitude:f64, deg:f64)-> (i32, i32) {
    
    let sunrise_angle:f64 = PI * deg / 180.0; /* sun angle at sunrise/set */

    /* get the day of year */
    let day_of_year:i32 = hdate_get_day_of_year (day, month, year);
    //println!("day_of_year = {}", day_of_year);

    /* location of sun in yearly cycle in radians */
    /* get radians of sun orbit around erth =) */
    let gama:f64 =  2.0 * PI * ((day_of_year as f64) - 1.0) / 365.0;

    /* difference between sun noon and clock noon */
    /* get the diff betwen suns clock and wall clock in minutes */
    let eqtime:f64 = 229.18 * (0.000075 + 0.001868 * gama.cos()
            - 0.032077 * gama.sin() - 0.014615 * ((2.0 * gama).cos())
            - 0.040849 * ((2.0 * gama).sin()) );

    //println!("Equation of time (min): {}", eqtime);

    /* sun declination */
    /* calculate suns declination at the equator in radians */
    let decl:f64 = 0.006918 - 0.399912 * gama.cos() + 0.070257 * gama.sin()
            - 0.006758 * ((2.0 * gama).cos()) + 0.000907 * ((2.0 * gama).sin())
            - 0.002697 * ((3.0 * gama).cos()) + 0.00148 * ((3.0 * gama).sin());


    // println!("Declination (radians): {}", decl);

    /* we use radians, ratio is 2pi/360 */
    let latitude:f64 = PI * latitude / 180.0;

    /* solar hour angle */
    /* the sun real time diff from noon at sunset/rise in radians */
//    errno = 0;
    let mut ha:f64 = (sunrise_angle.cos() / (latitude.cos() * decl.cos()) - latitude.tan() * 
            decl.tan()).acos();

    /* check for too high altitudes and return negative values */
//    if (errno == EDOM)
//    {
//            *sunrise = -720;
//            *sunset = -720;
//            return;
//    }

    /* we use minutes, ratio is 1440min/2pi */
    ha = 720.0 * ha / PI;
    //println!("Hour angle (degrees): {}", ha/60.0);

    /* get sunset/rise times in utc wall clock in minutes from 00:00 time */
    let sunrise:i32 = (720.0 - 4.0 * longitude - ha - eqtime).round() as i32;
    let sunset:i32 =  (720.0 - 4.0 * longitude + ha - eqtime).round() as i32;

    (sunrise, sunset)
}

fn build_ui(application: &gtk::Application) {
    let builder = Builder::from_file("example.glade");

    let window: Window = builder.get_object("window1").expect("Couldn't get window");
    let sb1: SpinButton = builder.get_object("sb1").expect("Couldn't get entry");
    let sb2: SpinButton = builder.get_object("sb2").expect("Couldn't get entry");
    let sb3: SpinButton = builder.get_object("sb3").expect("Couldn't get entry");
    let cal: Calendar = builder.get_object("cal").expect("Couldn't get entry");
    let b1: Button = builder.get_object("button1").expect("Couldn't get entry");
    
    window.set_application(Some(application));
    window.set_title("Sunrise Calculator");
   /* 
    window.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(true)
        });
*/
    let sb1_clone = sb1.clone();
    let sb2_clone = sb2.clone();
    let sb3_clone = sb3.clone();
    let cal_clone = cal.clone();
    b1.connect_clicked (move |_| {
        let userdate = cal_clone.get_date();
        let (year, month, day) = userdate;  //month is zero based???
        let month = month + 1;
        let latitude = sb1_clone.get_value();
        let longitude = sb2_clone.get_value();
        let tz_offset = sb3_clone.get_value();
        let (sunrise, sunset) = hdate_get_utc_sun_time_deg (
            day as i32,
            month as i32,
            year as i32,
            latitude,
            longitude,
            90.833
        );
        let y = year as i32;
        let hour = 3600;
        let tz = FixedOffset::east((tz_offset.trunc()) as i32 * hour);
        let sunrise_utc = Utc.ymd(y, month, day).and_hms(0, 0, 0) + Duration::minutes(sunrise as i64);
        let sunset_utc = Utc.ymd(y, month, day).and_hms(0, 0, 0) + Duration::minutes(sunset as i64);
        println!("sunrise_local  = {}", sunrise_utc.with_timezone(&tz).format("%A, %-d %B, %C%y, %r").to_string());
        println!("sunset_local  = {}", sunset_utc.with_timezone(&tz).format("%A, %-d %B, %C%y, %r").to_string());

    });
    
    window.show_all();
}

fn main() {
    let application = gtk::Application::new(
        Some("com.test.app"),
        Default::default(),
    )
    .expect("Initialization failed...");

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run(&args().collect::<Vec<_>>());
}


