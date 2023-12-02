import colorsys

RGB_COLORS = 32

print("RGB Colors: {}".format(RGB_COLORS))
for i in range(RGB_COLORS):
    rgb = colorsys.hsv_to_rgb((i * (255 / RGB_COLORS)) / 255, 1, 1)
    print("\x1b[48;2;{};{};{}m  ".format(int(rgb[0] * 255), int(rgb[1] * 255), int(rgb[2] * 255)), end="")

print("\x1b[0m")
