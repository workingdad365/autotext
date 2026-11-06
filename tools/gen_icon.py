"""AutoText 아이콘 생성 스크립트 (assets/autotext.ico, assets/autotext.png)"""
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

S = 1024
ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "assets"


def lerp(a, b, t):
    return tuple(int(a[i] + (b[i] - a[i]) * t) for i in range(len(a)))


def render() -> Image.Image:
    # 대각선 그라데이션 배경
    top, bottom = (59, 130, 246), (88, 28, 220)
    grad = Image.new("RGBA", (S, S))
    px = grad.load()
    for y in range(S):
        for x in range(S):
            px[x, y] = lerp(top, bottom, (x + y) / (2 * S)) + (255,)

    mask = Image.new("L", (S, S), 0)
    ImageDraw.Draw(mask).rounded_rectangle((40, 40, S - 40, S - 40), radius=210, fill=255)
    img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    img.paste(grad, (0, 0), mask)

    d = ImageDraw.Draw(img)

    # 키캡 모양
    d.rounded_rectangle((190, 210, 834, 814), radius=110, fill=(255, 255, 255, 60))
    d.rounded_rectangle((220, 230, 804, 760), radius=95, fill=(255, 255, 255, 255))

    # 글자 A
    font = None
    for name in ("segoeuib.ttf", "arialbd.ttf"):
        try:
            font = ImageFont.truetype(f"C:/Windows/Fonts/{name}", 470)
            break
        except OSError:
            continue
    if font is None:
        font = ImageFont.load_default()
    d.text((470, 500), "A", font=font, fill=(67, 56, 202, 255), anchor="mm")

    # 번개 (자동 입력 표시)
    bolt = [(700, 120), (560, 420), (660, 420), (590, 640), (820, 330), (715, 330), (790, 120)]
    shadow = [(x + 14, y + 14) for x, y in bolt]
    d.polygon(shadow, fill=(30, 20, 90, 110))
    d.polygon(bolt, fill=(250, 204, 21, 255), outline=(180, 120, 0, 255))
    return img


def main():
    OUT.mkdir(exist_ok=True)
    big = render()
    big.resize((256, 256), Image.LANCZOS).save(OUT / "autotext.png")
    sizes = [16, 20, 24, 32, 40, 48, 64, 128, 256]
    big.resize((256, 256), Image.LANCZOS).save(
        OUT / "autotext.ico", sizes=[(s, s) for s in sizes]
    )
    print("generated", OUT)


if __name__ == "__main__":
    main()
