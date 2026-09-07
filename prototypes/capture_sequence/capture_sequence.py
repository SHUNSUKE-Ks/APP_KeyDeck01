"""
連続キャプチャ（capture_sequence）単体動作テスト用プロトタイプ。
KeyDeckのパネルボタン用Action「action.capture_sequence」の実現可能性検証。

必要ライブラリ: mss, pyautogui
    pip install mss pyautogui

実行:
    python capture_sequence.py
"""

import tkinter as tk
from tkinter import filedialog, messagebox
import time
import os

import mss
import mss.tools
import pyautogui


class RegionSelector:
    """全画面オーバーレイでドラッグ矩形選択し、(left, top, width, height) を返す。"""

    def __init__(self, root):
        self.result = None
        self.start_x = None
        self.start_y = None
        self.rect_id = None

        self.overlay = tk.Toplevel(root)
        self.overlay.attributes("-fullscreen", True)
        self.overlay.attributes("-alpha", 0.3)
        self.overlay.configure(bg="black")
        self.overlay.attributes("-topmost", True)

        self.canvas = tk.Canvas(self.overlay, cursor="cross", bg="gray")
        self.canvas.pack(fill=tk.BOTH, expand=True)

        self.canvas.bind("<ButtonPress-1>", self.on_press)
        self.canvas.bind("<B1-Motion>", self.on_drag)
        self.canvas.bind("<ButtonRelease-1>", self.on_release)
        self.overlay.bind("<Escape>", self.on_cancel)

        self.overlay.grab_set()
        self.overlay.wait_window()

    def on_press(self, event):
        self.start_x, self.start_y = event.x, event.y
        self.rect_id = self.canvas.create_rectangle(
            self.start_x, self.start_y, self.start_x, self.start_y,
            outline="red", width=2
        )

    def on_drag(self, event):
        self.canvas.coords(self.rect_id, self.start_x, self.start_y, event.x, event.y)

    def on_release(self, event):
        x0, x1 = sorted((self.start_x, event.x))
        y0, y1 = sorted((self.start_y, event.y))
        if x1 - x0 > 2 and y1 - y0 > 2:
            self.result = (x0, y0, x1 - x0, y1 - y0)
        self.overlay.destroy()

    def on_cancel(self, event):
        self.result = None
        self.overlay.destroy()


class CaptureSequenceApp:
    def __init__(self, root):
        self.root = root
        self.root.title("連続キャプチャ")
        self.region = None
        self.save_dir = None

        tk.Label(root, text="スクリーンショット数").grid(row=0, column=0, sticky="w", padx=8, pady=4)
        self.count_var = tk.StringVar(value="10")
        vcmd = (root.register(self.validate_digits), "%P")
        tk.Entry(root, textvariable=self.count_var, validate="key", validatecommand=vcmd).grid(
            row=0, column=1, padx=8, pady=4
        )

        tk.Label(root, text="保存名（連番は自動付与）").grid(row=1, column=0, sticky="w", padx=8, pady=4)
        self.name_var = tk.StringVar(value="page")
        tk.Entry(root, textvariable=self.name_var).grid(row=1, column=1, padx=8, pady=4)

        tk.Label(root, text="保存先フォルダー").grid(row=2, column=0, sticky="w", padx=8, pady=4)
        self.dir_label = tk.Label(root, text="(未選択)", fg="gray")
        self.dir_label.grid(row=2, column=1, sticky="w", padx=8, pady=4)
        tk.Button(root, text="選択...", command=self.choose_dir).grid(row=2, column=2, padx=8, pady=4)

        tk.Label(root, text="撮影範囲").grid(row=3, column=0, sticky="w", padx=8, pady=4)
        self.region_label = tk.Label(root, text="(未選択)", fg="gray")
        self.region_label.grid(row=3, column=1, sticky="w", padx=8, pady=4)
        tk.Button(root, text="範囲選択...", command=self.choose_region).grid(row=3, column=2, padx=8, pady=4)

        tk.Button(root, text="OK（5秒後に開始）", command=self.start).grid(
            row=4, column=0, columnspan=3, pady=12
        )
        tk.Label(
            root,
            text="停止: マウスを画面の四隅いずれかへ動かす（PyAutoGUIの標準フェイルセーフ）",
            fg="gray",
        ).grid(row=5, column=0, columnspan=3, sticky="w", padx=8)

    def validate_digits(self, value):
        return value == "" or value.isdigit()

    def choose_dir(self):
        d = filedialog.askdirectory()
        if d:
            self.save_dir = d
            self.dir_label.config(text=d, fg="black")

    def choose_region(self):
        self.root.withdraw()
        time.sleep(0.3)
        selector = RegionSelector(self.root)
        self.root.deiconify()
        if selector.result:
            self.region = selector.result
            self.region_label.config(text=str(self.region), fg="black")

    def start(self):
        if not self.count_var.get().isdigit() or int(self.count_var.get()) <= 0:
            messagebox.showerror("エラー", "スクリーンショット数は1以上の半角数字で入力してください。")
            return
        if not self.save_dir:
            messagebox.showerror("エラー", "保存先フォルダーを選択してください。")
            return
        if not self.region:
            messagebox.showerror("エラー", "撮影範囲を選択してください。")
            return

        count = int(self.count_var.get())
        name = self.name_var.get().strip() or "page"

        self.root.withdraw()
        left, top, width, height = self.region

        for i in range(5, 0, -1):
            print(f"開始まで {i} 秒... 対象ウィンドウをアクティブにしてください")
            time.sleep(1)

        taken = 0
        stopped_reason = None
        try:
            with mss.mss() as sct:
                monitor = {"left": left, "top": top, "width": width, "height": height}
                for i in range(count):
                    img = sct.grab(monitor)
                    filename = os.path.join(self.save_dir, f"{name}{i:03d}.png")
                    mss.tools.to_png(img.rgb, img.size, output=filename)
                    print(f"保存: {filename}")
                    taken += 1

                    pyautogui.press("right")
                    time.sleep(0.5)
        except pyautogui.FailSafeException:
            stopped_reason = "マウスが画面端に移動されたため停止しました（フェイルセーフ）"
        except KeyboardInterrupt:
            stopped_reason = "Ctrl+Cにより停止しました"

        self.root.deiconify()
        if stopped_reason:
            print(f"中断: {stopped_reason}（{taken}/{count}枚まで完了）")
            messagebox.showwarning("中断", f"{stopped_reason}\n{taken}/{count}枚まで完了しています。")
        else:
            print("完了")
            messagebox.showinfo("完了", f"{count}枚のキャプチャが完了しました。")


if __name__ == "__main__":
    root = tk.Tk()
    app = CaptureSequenceApp(root)
    root.mainloop()
