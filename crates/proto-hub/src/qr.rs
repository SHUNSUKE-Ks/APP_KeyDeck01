//! D12: 接続URLをQRコード(SVG)として表示する。ユーザーが手でURL/tokenを入力せず、
//! ブラウザの `/` ランディングページに出るQRをスマホのカメラで読み取れるようにする。

use qrcode::render::svg;
use qrcode::QrCode;

pub fn svg_for_url(url: &str) -> Result<String, qrcode::types::QrError> {
    let code = QrCode::new(url.as_bytes())?;
    let svg = code
        .render()
        .min_dimensions(220, 220)
        .dark_color(svg::Color("#101526"))
        .light_color(svg::Color("#ffffff"))
        .build();
    Ok(svg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_svg_for_a_url() {
        let svg = svg_for_url("http://192.168.1.5:8770/kb?half=left&token=abc").unwrap();
        assert!(svg.contains("<svg"));
    }
}
