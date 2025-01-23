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
    mode: Mode,
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand)]
enum Mode {
    Seuil(OptsSeuil),
    Palette(OptsPalette),
    Bayer(OptsBayer),
    DiffusionErreurMonochrome(OptsDiffusionErreur),
    DiffusionErreurPalette(OptsDiffusionErreurPalette),
    DiffusionErreurFloydSteinberg(OptsDiffusionErreurFloydSteinberg),
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
    couleur2: String,
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="palette")]
/// Rendu de l’image avec une palette contenant un nombre limité de couleurs
struct OptsPalette {

    /// le nombre de couleurs à utiliser, dans la liste [NOIR, BLANC, ROUGE, VERT, BLEU, JAUNE, CYAN, MAGENTA]
    #[argh(option)]
    n_couleurs: usize,
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="bayer")]
/// Rendu de l’image par tramage ordonné de Bayer.
struct OptsBayer {
    /// ordre de la matrice de Bayer
    #[argh(option)]
    ordre: u32,
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="diffusion_erreur_monochrome")]
/// Diffusion d'erreur pour un rendu monochrome.
struct OptsDiffusionErreur {
    // Aucun paramètre spécifique à cette méthode de diffusion pour l'instant
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="diffusion_erreur_palette")]
/// Diffusion d'erreur pour un rendu utilisant une palette réduite de couleurs.
struct OptsDiffusionErreurPalette {

    /// le nombre de couleurs à utiliser, dans la liste [NOIR, BLANC, ROUGE, VERT, BLEU, JAUNE, CYAN, MAGENTA]
    #[argh(option)]
    n_couleurs: usize,
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="diffusion_erreur_floyd_steinberg")]
/// Diffusion d'erreur avec la méthode Floyd-Steinberg.
struct OptsDiffusionErreurFloydSteinberg {
    // Aucune option particulière ici
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
}

fn white_pixel_1_out_of_2(rgb_image: &mut image::RgbImage) {
    for (x, _, pixel) in rgb_image.enumerate_pixels_mut() {
        if x % 2 == 0 {
            *pixel = WHITE;
        }
    }
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
}

fn diffusion_erreur_monochrome(rgb_image: &mut image::RgbImage) {
    let mut erreur_image = vec![vec![0.0; rgb_image.width() as usize]; rgb_image.height() as usize];

    for y in 0..rgb_image.height() {
        for x in 0..rgb_image.width() {
            let pixel = rgb_image.get_pixel_mut(x, y);
            let luminance = get_luminance(pixel) / 255.0;

            let nouvelle_couleur = if luminance + erreur_image[y as usize][x as usize] > 0.5 {
                *pixel = WHITE;
                1.0
            } else {
                *pixel = BLACK;
                0.0
            };

            let erreur = luminance + erreur_image[y as usize][x as usize] - nouvelle_couleur;

            if x + 1 < rgb_image.width() {
                erreur_image[y as usize][(x + 1) as usize] += erreur * 0.5;
            }

            if y + 1 < rgb_image.height() {
                erreur_image[(y + 1) as usize][x as usize] += erreur * 0.5;
            }
        }
    }
}

fn diffusion_erreur_palette(rgb_image: &mut image::RgbImage, palette: Vec<image::Rgb<u8>>) {
    let mut erreur_image = vec![vec![[0.0; 3]; rgb_image.width() as usize]; rgb_image.height() as usize];

    for y in 0..rgb_image.height() {
        for x in 0..rgb_image.width() {
            let pixel = rgb_image.get_pixel_mut(x, y);
            let original_color = [
                pixel[0] as f32 + erreur_image[y as usize][x as usize][0],
                pixel[1] as f32 + erreur_image[y as usize][x as usize][1],
                pixel[2] as f32 + erreur_image[y as usize][x as usize][2],
            ];

            let mut closest_color = palette[0];
            let mut min_distance = f64::MAX;

            for &color in &palette {
                let distance = distance_eucli_btw_colors(
                    image::Rgb([original_color[0] as u8, original_color[1] as u8, original_color[2] as u8]),
                    color,
                );

                if distance < min_distance {
                    min_distance = distance;
                    closest_color = color;
                }
            }

            *pixel = closest_color;

            let erreur = [
                original_color[0] - closest_color[0] as f32,
                original_color[1] - closest_color[1] as f32,
                original_color[2] - closest_color[2] as f32,
            ];

            if x + 1 < rgb_image.width() {
                for c in 0..3 {
                    erreur_image[y as usize][(x + 1) as usize][c] += erreur[c] * 0.5;
                }
            }

            if y + 1 < rgb_image.height() {
                for c in 0..3 {
                    erreur_image[(y + 1) as usize][x as usize][c] += erreur[c] * 0.5;
                }
            }
        }
    }
}

fn diffusion_erreur_floyd_steinberg(rgb_image: &mut image::RgbImage) {
    let mut erreur_image = vec![vec![0.0; rgb_image.width() as usize]; rgb_image.height() as usize];

    for y in 0..rgb_image.height() {
        for x in 0..rgb_image.width() {
            let pixel = rgb_image.get_pixel_mut(x, y);
            let luminance = get_luminance(pixel) / 255.0 + erreur_image[y as usize][x as usize];

            let nouvelle_couleur = if luminance > 0.5 {
                *pixel = WHITE;
                1.0
            } else {
                *pixel = BLACK;
                0.0
            };

            let erreur = luminance - nouvelle_couleur;

            if x + 1 < rgb_image.width() {
                erreur_image[y as usize][(x + 1) as usize] += erreur * 7.0 / 16.0;
            }

            if y + 1 < rgb_image.height() {
                if x > 0 {
                    erreur_image[(y + 1) as usize][(x - 1) as usize] += erreur * 3.0 / 16.0;
                }
                erreur_image[(y + 1) as usize][x as usize] += erreur * 5.0 / 16.0;
                if x + 1 < rgb_image.width() {
                    erreur_image[(y + 1) as usize][(x + 1) as usize] += erreur * 1.0 / 16.0;
                }
            }
        }
    }
}

fn parse_couleur(couleur: &str) -> Option<image::Rgb<u8>> {
    match couleur.to_uppercase().as_str() {
        "BLACK" => Some(BLACK),
        "WHITE" => Some(WHITE),
        "RED" => Some(RED),
        "GREEN" => Some(GREEN),
        "BLUE" => Some(BLUE),
        "YELLOW" => Some(YELLOW),
        "CYAN" => Some(CYAN),
        "MAGENTA" => Some(MAGENTA),
        "GREY" => Some(GREY),
        _ => None,
    }
}

fn main() -> Result<(), ImageError> {
    let args: DitherArgs = argh::from_env();

    let path_in = args.input;
    let mut img: DynamicImage = ImageReader::open(path_in)?.decode()?;
    let mut rgb_image = img.to_rgb8();
    
    let palette: Vec<image::Rgb<u8>> = vec![BLACK, WHITE, RED, GREEN, BLUE, YELLOW, CYAN, MAGENTA];

    match args.mode {
        Mode::Seuil(opts) => {
            let seuil = opts.seuil.unwrap_or(127.5);
            let couleur1 = parse_couleur(&opts.couleur1).unwrap_or(WHITE);
            let couleur2 = parse_couleur(&opts.couleur2).unwrap_or(BLACK);
            apply_threshold_seuillage(&mut rgb_image, couleur1, couleur2);
        }
        Mode::Palette(opts) => {
            if opts.n_couleurs == 0 {
                println!("Erreur : Le nombre de couleurs doit être supérieur à 0.");
                return Ok(());
            }
            apply_distance_eucli(&mut rgb_image, palette[..opts.n_couleurs].to_vec());
        }
        Mode::Bayer(opts) => {
            apply_matrice_bayer(&mut rgb_image, opts.ordre);
        }
        Mode::DiffusionErreurMonochrome(_) => {
            diffusion_erreur_monochrome(&mut rgb_image);
        }
        Mode::DiffusionErreurPalette(_) => {
            let palette: Vec<image::Rgb<u8>> = vec![BLACK, WHITE, RED, GREEN, BLUE, YELLOW, CYAN, MAGENTA];
            diffusion_erreur_palette(&mut rgb_image, palette);
        }
        Mode::DiffusionErreurFloydSteinberg(_) => {
            diffusion_erreur_floyd_steinberg(&mut rgb_image);
        }
    }

    let output_path = args.output.unwrap_or_else(|| "./output/out.png".to_string());
    rgb_image.save_with_format(&output_path, image::ImageFormat::Png)?;

    println!("Traitement terminé. Résultat enregistré dans {}", output_path);
    Ok(())
}

