from statistics import mean

from PIL import Image, ImageChops, ImageDraw

from energymonitor.devices import rpict, linky
from energymonitor.helpers.constants import MAX_POWER, MIN_POWER, LOGO, VERSION, FONT
from energymonitor.helpers.imaging import clear, add_text, add_bar


class Page:

    def __init__(self, size: (int, int)) -> None:
        self.im = Image.new(mode='1', size=size, color=0)

    def image(self) -> Image:
        return self.im


class LandingPage(Page):

    def __init__(self, size: (int, int)) -> None:
        super().__init__(size)
        self.im.paste(LOGO)
        self.im = ImageChops.offset(self.im, xoffset=0, yoffset=-5)
        version = f'v{VERSION}'
        (font_width, font_height) = FONT.getsize(version)
        ImageDraw.Draw(self.im).text(xy=((LOGO.size[0] - font_width) // 2, LOGO.size[1] - font_height - 1),
                                     text=version, font=FONT, fill=255)


class RPICTPage(Page):

    def __init__(self, size: (int, int)) -> None:
        super().__init__(size)
        self.max_l1_apparent_power = MIN_POWER
        self.max_l2_apparent_power = MIN_POWER
        self.max_l3_apparent_power = MIN_POWER

    def refresh(self, m: rpict.Measurements):
        # refresh state
        self.max_l1_apparent_power = max(self.max_l1_apparent_power, m.l1_apparent_power)
        self.max_l2_apparent_power = max(self.max_l2_apparent_power, m.l2_apparent_power)
        self.max_l3_apparent_power = max(self.max_l3_apparent_power, m.l3_apparent_power)
        # clear image
        clear(self.im)
        # draw line 1
        add_text(self.im, (0, 0), f'P1 {m.l1_apparent_power:4.0f}W')
        add_bar(self.im, (52, 0), m.l1_apparent_power / MAX_POWER, self.max_l1_apparent_power / MAX_POWER)
        # draw line 2
        add_text(self.im, (0, 8), f'P2 {m.l2_apparent_power:4.0f}W')
        add_bar(self.im, (52, 8), m.l2_apparent_power / MAX_POWER, self.max_l2_apparent_power / MAX_POWER)
        # draw line 3
        add_text(self.im, (0, 16), f'P3 {m.l3_apparent_power:4.0f}W')
        add_bar(self.im, (52, 16), m.l3_apparent_power / MAX_POWER, self.max_l3_apparent_power / MAX_POWER)
        # draw line 4
        total_apparent_power = (m.l1_apparent_power + m.l2_apparent_power + m.l3_apparent_power) / 1000
        add_text(self.im, (0, 24), f'= {total_apparent_power:4.1f}kW')
        avg_vrms = mean([m.l1_vrms, m.l2_vrms, m.l3_vrms])
        add_text(self.im, (87, 24), f'{avg_vrms:5.2f}V')


class LinkyPage(Page):

    def __init__(self, size: (int, int)) -> None:
        super().__init__(size)

    def refresh(self, m: linky.Measurements):
        # clear image
        clear(self.im)
        # draw line 1
        add_text(self.im, (0, 0), f'ID: {m.ADCO}')
        # draw line 2
        add_text(self.im, (0, 8), f'> HP {m.HCHP}W')
        # draw line 3
        add_text(self.im, (0, 16), f'  HC {m.HCHC}W')
