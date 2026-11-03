package top.cosas.rix;

import android.content.ClipData;
import android.content.ClipboardManager;
import android.content.Intent;
import android.graphics.Rect;
import android.net.Uri;
import android.os.Bundle;
import android.view.HapticFeedbackConstants;
import android.view.WindowInsets;
import android.view.WindowInsetsController;

import java.io.File;
import java.util.List;

import androidx.activity.OnBackPressedCallback;
import androidx.core.content.FileProvider;

import com.google.androidgamesdk.GameActivity;

public class MainActivity extends GameActivity {
    static {
        System.loadLibrary("rix_mobile");
    }

    /** True when the game takes Back: it pauses the run ([079], `src/game/pause.rs`). */
    private static native boolean onBackInPlay();

    /** Called by the game over JNI ([081], `src/haptics.rs`): 0 light, 1 medium, 2 heavy. */
    public void haptic(int kind) {
        int[] effects = {
            HapticFeedbackConstants.KEYBOARD_TAP,
            HapticFeedbackConstants.VIRTUAL_KEY,
            HapticFeedbackConstants.LONG_PRESS,
        };
        runOnUiThread(() -> getWindow().getDecorView().performHapticFeedback(effects[kind]));
    }

    /**
     * The share sheet for a text and, when `picturePath` is not null, a PNG in the files
     * directory ([067], [095], `src/share.rs`).
     */
    public void share(String text, String picturePath) {
        Intent intent = new Intent(Intent.ACTION_SEND).putExtra(Intent.EXTRA_TEXT, text);
        if (picturePath == null) {
            intent.setType("text/plain");
        } else {
            Uri picture = FileProvider.getUriForFile(
                    this, getPackageName() + ".share", new File(picturePath));
            intent.setType("image/png")
                    .putExtra(Intent.EXTRA_STREAM, picture)
                    .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION);
            // The sheet shows the picture as its preview.
            intent.setClipData(ClipData.newRawUri(null, picture));
        }
        runOnUiThread(() -> startActivity(Intent.createChooser(intent, null)));
    }

    /** The clipboard's text, or null ([067], `src/share.rs`). */
    public String pasteText() {
        ClipData clip = clipboard.getPrimaryClip();
        if (clip == null || clip.getItemCount() == 0) {
            return null;
        }
        return clip.getItemAt(0).coerceToText(this).toString();
    }

    /** Made on the UI thread: the game thread reads it. */
    private ClipboardManager clipboard;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        clipboard = getSystemService(ClipboardManager.class);
        // Back would destroy the activity under the running native app ([023]): in play it
        // pauses the run; elsewhere it moves the task to the background, as Home does.
        getOnBackPressedDispatcher().addCallback(this, new OnBackPressedCallback(true) {
            @Override
            public void handleOnBackPressed() {
                if (!onBackInPlay()) {
                    moveTaskToBack(true);
                }
            }
        });
        // Swipes from the side edges draw lines, not Back ([079]). The 200 dp per edge limit
        // does not apply while the bars are hidden with BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE.
        getWindow().getDecorView().addOnLayoutChangeListener(
                (view, left, top, right, bottom, oldLeft, oldTop, oldRight, oldBottom) ->
                        view.setSystemGestureExclusionRects(
                                List.of(new Rect(0, 0, right - left, bottom - top))));
    }

    @Override
    public void onWindowFocusChanged(boolean hasFocus) {
        super.onWindowFocusChanged(hasFocus);

        if (hasFocus) {
            hideSystemUi();
        }
    }

    /** Sticky immersive: a swipe from a bar edge shows the bars for a moment ([079]). */
    private void hideSystemUi() {
        // Edge to edge on API 31 to 34; API 35 and later do it for apps that target 35 or more.
        getWindow().setDecorFitsSystemWindows(false);
        WindowInsetsController controller = getWindow().getInsetsController();
        controller.hide(WindowInsets.Type.systemBars());
        controller.setSystemBarsBehavior(
                WindowInsetsController.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE);
    }
}
