import colorsys

RGB_COLORS = 32

print("RGB24 rainbow:")
for i in range(RGB_COLORS):
    rgb = colorsys.hsv_to_rgb((i * (255 / RGB_COLORS)) / 255, 1, 1)
    print("\x1b[0;48;2;{};{};{}m  ".format(int(rgb[0] * 255), int(rgb[1] * 255), int(rgb[2] * 255)), end="")
print("\x1b[0m")

print("ANSI 8-bit colors:")
for y in range(16):
    for x in range(16):
        print("\x1b[48;5;{}m  ".format(y * 16 + x), end="")
    print("\x1b[0m")

print("ANSI 4-bit colors:")
for i in range(8):
    print("\x1b[4{}m  ".format(i), end="")
print("\x1b[0m")
for i in range(8):
    print("\x1b[10{}m  ".format(i), end="")
print("\x1b[0m")
