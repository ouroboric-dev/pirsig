package io.github.h0ffmann.pirsig

import android.graphics.Canvas
import android.graphics.ColorFilter
import android.graphics.Paint
import android.graphics.PixelFormat
import android.graphics.drawable.Drawable
import android.os.Bundle
import android.view.View
import androidx.activity.enableEdgeToEdge
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
    // The page sits between the system bars (and above the keyboard). Behind the
    // status bar shows the page's colour, behind the navigation bar the tab bar's,
    // as in the mobile design (res/values*/colors.xml, the design's tokens).
    val content = findViewById<View>(android.R.id.content)
    content.background = SystemBars(getColor(R.color.pirsig_bg), getColor(R.color.pirsig_panel))
    ViewCompat.setOnApplyWindowInsetsListener(content) { view, insets ->
      val bars = insets.getInsets(
        WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout() or WindowInsetsCompat.Type.ime()
      )
      view.setPadding(bars.left, bars.top, bars.right, bars.bottom)
      WindowInsetsCompat.CONSUMED
    }
  }
}

/** The top half in one colour, the bottom half in another: only the padded strips show. */
private class SystemBars(top: Int, bottom: Int) : Drawable() {
  private val top = Paint().apply { color = top }
  private val bottom = Paint().apply { color = bottom }

  override fun draw(canvas: Canvas) {
    val b = bounds
    val mid = b.exactCenterY()
    canvas.drawRect(b.left.toFloat(), b.top.toFloat(), b.right.toFloat(), mid, top)
    canvas.drawRect(b.left.toFloat(), mid, b.right.toFloat(), b.bottom.toFloat(), bottom)
  }

  override fun setAlpha(alpha: Int) {}

  override fun setColorFilter(colorFilter: ColorFilter?) {}

  @Deprecated("Deprecated in Java")
  override fun getOpacity() = PixelFormat.OPAQUE
}
