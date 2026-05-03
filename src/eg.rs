use embedded_graphics_core::{
    Drawable,
    draw_target::DrawTarget,
    geometry::{Dimensions, OriginDimensions, Point, Size},
    pixelcolor::PixelColor,
    primitives::Rectangle,
};

use crate::{QrMatrix, Version};

#[cfg_attr(docsrs, doc(cfg(feature = "embedded-graphics")))]
/// A feature-gated `embedded-graphics` adapter for [`QrMatrix`].
///
/// The drawable is positioned at the origin and can be configured with module
/// colors, module size, and quiet-zone border width.
#[derive(Debug, Clone, Copy)]
pub struct QrDrawable<'a, T: Version, C: PixelColor + Copy> {
    matrix: &'a QrMatrix<T>,
    dark_color: C,
    light_color: C,
    module_size: u32,
    border: u32,
}

impl<'a, T: Version, C: PixelColor + Copy> QrDrawable<'a, T, C> {
    /// Creates a drawable wrapper around a [`QrMatrix`].
    ///
    /// The default configuration uses a module size of `1` pixel and a quiet
    /// zone border of `0` modules.
    pub fn new(matrix: &'a QrMatrix<T>, dark_color: C, light_color: C) -> Self {
        Self {
            matrix,
            dark_color,
            light_color,
            module_size: 1,
            border: 0,
        }
    }

    /// Sets the pixel size of each QR module.
    pub fn with_module_size(mut self, module_size: u32) -> Self {
        assert!(module_size > 0, "module size must be greater than zero");
        self.module_size = module_size;
        self
    }

    /// Sets the quiet-zone border width in modules.
    pub fn with_border(mut self, border: u32) -> Self {
        self.border = border;
        self
    }

    /// Returns the wrapped matrix.
    pub fn matrix(&self) -> &'a QrMatrix<T> {
        self.matrix
    }

    /// Returns the configured dark module color.
    pub fn dark_color(&self) -> C {
        self.dark_color
    }

    /// Returns the configured light module color.
    pub fn light_color(&self) -> C {
        self.light_color
    }

    /// Returns the configured pixel size of each module.
    pub fn module_size(&self) -> u32 {
        self.module_size
    }

    /// Returns the configured quiet-zone border width in modules.
    pub fn border(&self) -> u32 {
        self.border
    }

    fn total_modules(&self) -> u32 {
        (self.matrix.width() as u32).saturating_add(self.border.saturating_mul(2))
    }

    fn pixel_size(&self) -> u32 {
        self.total_modules().saturating_mul(self.module_size)
    }

    fn module_origin(&self, x: usize, y: usize) -> Option<Point> {
        let module_x = (x as u32)
            .saturating_add(self.border)
            .saturating_mul(self.module_size);
        let module_y = (y as u32)
            .saturating_add(self.border)
            .saturating_mul(self.module_size);

        Some(Point::new(
            i32::try_from(module_x).ok()?,
            i32::try_from(module_y).ok()?,
        ))
    }
}

impl<T: Version, C: PixelColor + Copy> OriginDimensions for QrDrawable<'_, T, C> {
    fn size(&self) -> Size {
        let size = self.pixel_size();
        Size::new(size, size)
    }
}

impl<T: Version, C: PixelColor + Copy> Drawable for QrDrawable<'_, T, C> {
    type Color = C;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        target.fill_solid(&self.bounding_box(), self.light_color)?;

        for (x, y, dark) in self.matrix.iter() {
            if !dark {
                continue;
            }

            if let Some(top_left) = self.module_origin(x, y) {
                let area = Rectangle::new(top_left, Size::new(self.module_size, self.module_size));
                target.fill_solid(&area, self.dark_color)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use core::convert::Infallible;

    use embedded_graphics_core::{
        Drawable, Pixel,
        draw_target::DrawTarget,
        geometry::{OriginDimensions, Point, Size},
        pixelcolor::BinaryColor,
    };

    use super::QrDrawable;
    use crate::{QrBuilder, Version1};

    struct TestTarget {
        size: Size,
        pixels: std::vec::Vec<BinaryColor>,
    }

    impl TestTarget {
        fn with_size(size: Size) -> Self {
            Self {
                size,
                pixels: std::vec![BinaryColor::Off; (size.width * size.height) as usize],
            }
        }

        fn get(&self, x: u32, y: u32) -> BinaryColor {
            self.pixels[(y * self.size.width + x) as usize]
        }
    }

    impl DrawTarget for TestTarget {
        type Color = BinaryColor;
        type Error = Infallible;

        fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
        where
            I: IntoIterator<Item = Pixel<Self::Color>>,
        {
            for Pixel(coord, color) in pixels {
                if let Ok((x, y)) = <(u32, u32)>::try_from(coord) {
                    if x < self.size.width && y < self.size.height {
                        let index = (y * self.size.width + x) as usize;
                        self.pixels[index] = color;
                    }
                }
            }

            Ok(())
        }
    }

    impl OriginDimensions for TestTarget {
        fn size(&self) -> Size {
            self.size
        }
    }

    #[test]
    fn drawable_defaults_match_qr_quiet_zone() {
        let matrix = QrBuilder::<Version1>::new().build(b"HELLO WORLD").unwrap();
        let drawable = QrDrawable::new(&matrix, BinaryColor::On, BinaryColor::Off);

        assert_eq!(drawable.module_size(), 1);
        assert_eq!(drawable.border(), 0);
        assert_eq!(drawable.size(), Size::new(21, 21));
    }

    #[test]
    fn qrmatrix_drawable_convenience_method_uses_defaults() {
        let matrix = QrBuilder::<Version1>::new().build(b"HELLO WORLD").unwrap();
        let drawable = matrix.into_drawable(BinaryColor::On, BinaryColor::Off);

        assert_eq!(drawable.size(), Size::new(21, 21));
    }

    #[test]
    fn drawable_renders_border_and_scaled_modules() {
        let matrix = QrBuilder::<Version1>::new().build(b"HELLO WORLD").unwrap();
        let drawable = QrDrawable::new(&matrix, BinaryColor::On, BinaryColor::Off)
            .with_module_size(2)
            .with_border(1);
        let mut target = TestTarget::with_size(drawable.size());

        drawable.draw(&mut target).unwrap();

        assert_eq!(target.get(0, 0), BinaryColor::Off);
        assert_eq!(target.get(1, 1), BinaryColor::Off);

        assert_eq!(target.get(2, 2), BinaryColor::On);
        assert_eq!(target.get(3, 3), BinaryColor::On);

        let white_module = Point::new(16, 16);
        assert_eq!(
            target.get(white_module.x as u32, white_module.y as u32),
            BinaryColor::Off
        );
        assert_eq!(drawable.matrix().get(7, 7), false);
    }
}
