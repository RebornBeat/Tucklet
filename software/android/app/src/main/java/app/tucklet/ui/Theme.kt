// Theme.kt — warm, calm keepsake palette (matches the iOS app / UX_SPEC).
// License: PolyForm Noncommercial 1.0.0
package app.tucklet.ui

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

object Brand {
    val accent = Color(0xFFB56650) // dusty terracotta
    val ink = Color(0xFF3B2C24)
    val paper = Color(0xFFFAF7F2)
    val muted = Color(0xFF8C7663)
}

private val LightColors = lightColorScheme(
    primary = Brand.accent,
    onPrimary = Color.White,
    background = Brand.paper,
    surface = Color.White,
    onBackground = Brand.ink,
    onSurface = Brand.ink,
)

@Composable
fun TuckletTheme(content: @Composable () -> Unit) {
    // Keep the warm light palette even in dark mode for brand consistency; a
    // full dark scheme can be added later.
    @Suppress("UNUSED_VARIABLE") val dark = isSystemInDarkTheme()
    MaterialTheme(colorScheme = LightColors, content = content)
}
