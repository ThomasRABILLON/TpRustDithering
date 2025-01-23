use argh::FromArgs;
use image::io::Reader as ImageReader;
use image::{DynamicImage, ImageError};
use rand::Rng;

#[derive(Debug, Clone, PartialEq, FromArgs)]
/// Convertit une image en monochrome ou vers une palette réduite de couleurs.
struct DitherArgs {
    /// le fichier d’entrée
    #[argh(positional)]
    input: String,

    /// le fichier de sortie (optionnel)
    #[argh(positional)]
    output: Option<String>,
    /// le mode d’opération
    #[argh(subcommand)]
    mode: Mode
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand)]
enum Mode {
    Seuil(OptsSeuil),
    Palette(OptsPalette),
    Bayer(OptsBayer)
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="seuil")]
/// Rendu de l’image par seuillage monochrome.
struct OptsSeuil {
    /// seuil de luminance pour le seuillage
    #[argh(option)]
    seuil: Option<f32>,

    /// couleur pour les pixels dont la luminance est supérieure au seuil
    #[argh(option)]
    couleur1: String,

    /// couleur pour les pixels dont la luminance est inférieure ou égale au seuil
    #[argh(option)]
    couleur2: String
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="palette")]
/// Rendu de l’image avec une palette contenant un nombre limité de couleurs
struct OptsPalette {

    /// le nombre de couleurs à utiliser, dans la liste [NOIR, BLANC, ROUGE, VERT, BLEU, JAUNE, CYAN, MAGENTA]
    #[argh(option)]
    n_couleurs: usize
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="bayer")]
/// Rendu de l’image par tramage ordonné de Bayer.
struct OptsBayer {
    /// ordre de la matrice de Bayer
    #[argh(option)]
    ordre: u32
}

const WHITE: image::Rgb<u8> = image::Rgb([255, 255, 255]);
const GREY: image::Rgb<u8> = image::Rgb([127, 127, 127]);
const BLACK: image::Rgb<u8> = image::Rgb([0, 0, 0]);
const BLUE: image::Rgb<u8> = image::Rgb([0, 0, 255]);
const RED: image::Rgb<u8> = image::Rgb([255, 0, 0]);
const GREEN: image::Rgb<u8> = image::Rgb([0, 255, 0]);
const YELLOW: image::Rgb<u8> = image::Rgb([255, 255, 0]);
const MAGENTA: image::Rgb<u8> = image::Rgb([255, 0, 255]);
const CYAN: image::Rgb<u8> = image::Rgb([0, 255, 255]);

fn distance_eucli_btw_colors(c1: image::Rgb<u8>, c2: image::Rgb<u8>) -> f64 {
    let r1 = c1[0] as f64;
    let g1 = c1[1] as f64;
    let b1 = c1[2] as f64;

    let r2 = c2[0] as f64;
    let g2 = c2[1] as f64;
    let b2 = c2[2] as f64;

    ((r1 - r2).powi(2) + (g1 - g2).powi(2) + (b1 - b2).powi(2)).sqrt()
}

fn apply_distance_eucli(rgb_image: &mut image::RgbImage, palette: Vec<image::Rgb<u8>>) {
    for (_x, _y, pixel) in rgb_image.enumerate_pixels_mut() {
        let mut min_distance = f64::MAX;
        let mut closest_color = palette[0];

        for color in &palette {
            let distance = distance_eucli_btw_colors(*pixel, *color);
            if distance < min_distance {
                min_distance = distance;
                closest_color = *color;
            }
        }
        *pixel = closest_color;
    }
    rgb_image
        .save_with_format("./output/output_palette.png", image::ImageFormat::Png)
        .unwrap();
}

fn white_pixel_1_out_of_2(rgb_image: &mut image::RgbImage) {
    for (x, _, pixel) in rgb_image.enumerate_pixels_mut() {
        if x % 2 == 0 {
            *pixel = WHITE;
        }
    }
    rgb_image
        .save_with_format("./output/1_pixel_blanc_sur_2.png", image::ImageFormat::Png)
        .unwrap();
}

fn get_luminance(pixel: &image::Rgb<u8>) -> f32 {
    let r = pixel[0] as f32;
    let g = pixel[1] as f32;
    let b = pixel[2] as f32;

    0.2126f32 * r + 0.7152f32 * g + 0.0722f32 * b
}

fn apply_threshold_seuillage(rgb_image: &mut image::RgbImage, couleur1: image::Rgb<u8>, couleur2: image::Rgb<u8>) {
    for (_x, _y, pixel) in rgb_image.enumerate_pixels_mut() {
        let luminance = get_luminance(pixel);

        if luminance > 127.5 {
            *pixel = couleur1;
        } else {
            *pixel = couleur2;
        }
    }
    rgb_image
        .save_with_format("./output/output_monochrome.png", image::ImageFormat::Png)
        .unwrap();
}

fn tramage_aleatoire(rgb_image: &mut image::RgbImage) {
    let mut rng = rand::thread_rng();
    let mut random = 0;
    for (x, y, pixel) in rgb_image.enumerate_pixels_mut() {
        random = rng.gen_range(0..255);
        if get_luminance(pixel) > random as f32 {
            *pixel = WHITE;
        } else {
            *pixel = BLACK;
        }
    }
    rgb_image
        .save_with_format("./output/output_tramage_aleatoire.png", image::ImageFormat::Png)
        .unwrap();
}

fn genere_matrice_bayer(ordre: u32) -> Vec<Vec<u32>> {
    if ordre == 0 {
        return vec![vec![0]];
    }

    let matrice_prec = genere_matrice_bayer(ordre - 1);
    let size = matrice_prec.len();
    let new_size = size * 2;
    let mut matrice = vec![vec![0; new_size]; new_size];

    for i in 0..size {
        for j in 0..size {
            let base_value = matrice_prec[i][j];
            matrice[i][j] = base_value * 4;
            matrice[i + size][j] = base_value * 4 + 2;
            matrice[i][j + size] = base_value * 4 + 3;
            matrice[i + size][j + size] = base_value * 4 + 1;
        }
    }

    matrice
}

fn apply_matrice_bayer(rgb_image: &mut image::RgbImage, ordre: u32) {
    let matrice = genere_matrice_bayer(ordre);
    let size = matrice.len() as u32;
    for (x, y, pixel) in rgb_image.enumerate_pixels_mut() {
        let i = x % size;
        let j = y % size;
        let seuil = matrice[i as usize][j as usize] * 255 / (size * size);
        if get_luminance(pixel) > seuil as f32 {
            *pixel = WHITE;
        } else {
            *pixel = BLACK;
        }
    }
    rgb_image
        .save_with_format("./output/output_tramage_bayer.png", image::ImageFormat::Png)
        .unwrap();
}

fn main() -> Result<(), ImageError> {
    let args: DitherArgs = argh::from_env();
    let path_in = args.input;
    let img: DynamicImage = ImageReader::open(path_in)?.decode()?;
    let rgb_image = img.to_rgb8();

    // Coordonnées du pixel (32, 52)
    let x = 32;
    let y = 52;

    // Vérifie si les coordonnées sont valides
    if x < rgb_image.width() && y < rgb_image.height() {
        // Récupère la couleur du pixel
        let pixel = rgb_image.get_pixel(x, y);
        println!("La couleur du pixel ({}, {}) est : {:?}", x, y, pixel);
    } else {
        println!(
            "Coordonnées ({}, {}) hors de l'image (dimensions : {}x{}).",
            x,
            y,
            rgb_image.width(),
            rgb_image.height()
        );
    }

    white_pixel_1_out_of_2(&mut rgb_image.clone());

    tramage_aleatoire(&mut rgb_image.clone());

    rgb_image.save_with_format("./output/output.png", image::ImageFormat::Png)?;

    match args.mode {
        Mode::Seuil(_) => {
            let opts = match args.mode {
                Mode::Seuil(opts) => opts,
                _ => unreachable!()
            };
            let mut couleur1 = WHITE;
            match opts.couleur1.as_str() {
                "BLACK" => couleur1 = BLACK,
                "WHITE" => couleur1 = WHITE,
                "RED" => couleur1 = RED,
                "GREEN" => couleur1 = GREEN,
                "BLUE" => couleur1 = BLUE,
                "YELLOW" => couleur1 = YELLOW,
                "CYAN" => couleur1 = CYAN,
                "MAGENTA" => couleur1 = MAGENTA,
                "GREY" => couleur1 = GREY,
                _ => couleur1 = WHITE,
            }    
            let mut couleur2 = BLACK;
            match opts.couleur2.as_str() {
                "BLACK" => couleur2 = BLACK,
                "WHITE" => couleur2 = WHITE,
                "RED" => couleur2 = RED,
                "GREEN" => couleur2 = GREEN,
                "BLUE" => couleur2 = BLUE,
                "YELLOW" => couleur2 = YELLOW,
                "CYAN" => couleur2 = CYAN,
                "MAGENTA" => couleur2 = MAGENTA,
                "GREY" => couleur2 = GREY,
                _ => couleur2 = BLACK,
            }

            apply_threshold_seuillage(&mut rgb_image.clone(), couleur1, couleur2);
        }
        Mode::Palette(opts) => {
            if opts.n_couleurs == 0 {
                println!("Le nombre de couleurs doit être supérieur à 0.");
                return Ok(());
            }

            let palette: Vec<image::Rgb<u8>> = vec![BLACK, WHITE, RED, GREEN, BLUE, YELLOW, CYAN, MAGENTA];
            apply_distance_eucli(&mut rgb_image.clone(), palette[..opts.n_couleurs].to_vec());
        }
        Mode::Bayer(opts) => {
            apply_matrice_bayer(&mut rgb_image.clone(), opts.ordre);
        }
    }

    Ok(())
}
