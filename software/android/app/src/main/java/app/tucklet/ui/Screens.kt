// Screens.kt
// The Compose UI: onboarding gate, Home, Library (unified on-phone + on-Tucklet
// with real thumbnails), item detail + restore, transfer sheet (estimate up
// front), and Settings. Plain-language states throughout (UX_SPEC).
//
// License: PolyForm Noncommercial 1.0.0
package app.tucklet.ui

import android.graphics.Bitmap
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import app.tucklet.core.TransferEstimator
import app.tucklet.protocol.*
import app.tucklet.store.AppViewModel

@Composable
fun TuckletApp(vm: AppViewModel, onStartPairing: () -> Unit) {
    val paired by vm.isPaired.collectAsState()
    if (!paired) {
        OnboardingScreen(vm, onStartPairing)
    } else {
        MainScaffold(vm)
    }
}

@Composable
private fun OnboardingScreen(vm: AppViewModel, onStartPairing: () -> Unit) {
    Column(
        Modifier.fillMaxSize().background(Brand.paper).padding(24.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        Icon(Icons.Filled.Favorite, null, tint = Brand.accent, modifier = Modifier.size(64.dp))
        Spacer(Modifier.height(16.dp))
        Text("Meet your Tucklet", fontSize = 26.sp, fontWeight = FontWeight.Bold, color = Brand.ink)
        Spacer(Modifier.height(8.dp))
        Text(
            "Bring it close and tap to connect. You'll only do this once — after that it just works whenever it's near.",
            color = Brand.muted,
        )
        Spacer(Modifier.height(32.dp))
        Button(onClick = onStartPairing, modifier = Modifier.fillMaxWidth()) {
            Text("Connect my Tucklet")
        }
        val err by vm.errorText.collectAsState()
        err?.let { Spacer(Modifier.height(12.dp)); Text(it, color = MaterialTheme.colorScheme.error) }
    }
}

private enum class Tab(val label: String, val icon: androidx.compose.ui.graphics.vector.ImageVector) {
    HOME("Home", Icons.Filled.Home),
    LIBRARY("Library", Icons.Filled.GridView),
    SETTINGS("Settings", Icons.Filled.Settings),
}

@Composable
private fun MainScaffold(vm: AppViewModel) {
    var tab by remember { mutableStateOf(Tab.HOME) }
    var detail by remember { mutableStateOf<MediaItem?>(null) }
    var transfer by remember { mutableStateOf<Pair<TransferKind, List<MediaItem>>?>(null) }

    LaunchedEffect(Unit) { vm.connect(); vm.loadLibrary() }

    Scaffold(
        containerColor = Brand.paper,
        bottomBar = {
            NavigationBar(containerColor = androidx.compose.ui.graphics.Color.White) {
                Tab.entries.forEach { t ->
                    NavigationBarItem(
                        selected = tab == t,
                        onClick = { tab = t },
                        icon = { Icon(t.icon, t.label) },
                        label = { Text(t.label) },
                    )
                }
            }
        }
    ) { pad ->
        Box(Modifier.padding(pad)) {
            when (tab) {
                Tab.HOME -> HomeScreen(vm)
                Tab.LIBRARY -> LibraryScreen(vm, onOpen = { detail = it }, onTransfer = { k, items -> transfer = k to items })
                Tab.SETTINGS -> SettingsScreen(vm)
            }
        }
    }

    detail?.let { item ->
        ItemDetailSheet(vm, item, onClose = { detail = null })
    }
    transfer?.let { (kind, items) ->
        TransferSheet(vm, kind, items, onClose = { transfer = null })
    }
}

@Composable
private fun HomeScreen(vm: AppViewModel) {
    val status by vm.status.collectAsState()
    val conn by vm.connection.collectAsState()
    val lastOffload by vm.lastOffloadIds.collectAsState()
    Column(Modifier.fillMaxSize().padding(16.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
        Card { Column(Modifier.padding(20.dp).fillMaxWidth(), horizontalAlignment = Alignment.CenterHorizontally) {
            Icon(Icons.Filled.Album, null, tint = Brand.accent, modifier = Modifier.size(44.dp))
            Spacer(Modifier.height(8.dp))
            val s = status
            if (s != null) {
                Text("${s.batteryPercent}%${if (s.charging) " · charging" else ""}", fontWeight = FontWeight.Bold, color = Brand.ink)
                Text("${vm.byteFormat(s.freeBytes)} free of ${vm.byteFormat(s.totalBytes)}", color = Brand.muted)
            } else if (conn == app.tucklet.store.AppRepository.Conn.CONNECTING) {
                Text("Finding your Tucklet…", color = Brand.muted)
            } else {
                Text("Bring your Tucklet near", color = Brand.muted)
            }
        } }
        if (lastOffload.isNotEmpty()) {
            Card(colors = CardDefaults.cardColors(containerColor = Brand.accent.copy(alpha = 0.08f))) {
                Row(Modifier.padding(16.dp).fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                    Icon(Icons.Filled.CheckCircle, null, tint = Brand.accent)
                    Spacer(Modifier.width(8.dp))
                    Text("Backed up ${lastOffload.size} photos", color = Brand.ink)
                    Spacer(Modifier.weight(1f))
                    TextButton(onClick = { vm.undoLastOffload() }) { Text("Undo") }
                }
            }
        }
        val pending = vm.pendingBackupCount()
        Card { Row(Modifier.padding(16.dp).fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
            Icon(if (pending == 0) Icons.Filled.Verified else Icons.Filled.Upload, null, tint = Brand.accent)
            Spacer(Modifier.width(8.dp))
            Text(if (pending == 0) "Everything's backed up" else "$pending photos waiting to back up", color = Brand.ink)
        } }
    }
}

@Composable
private fun LibraryScreen(
    vm: AppViewModel,
    onOpen: (MediaItem) -> Unit,
    onTransfer: (TransferKind, List<MediaItem>) -> Unit,
) {
    val manifest by vm.manifest.collectAsState()
    val onPhone by vm.onPhoneItems.collectAsState()
    val selection = remember { mutableStateListOf<String>() }
    val groups = remember(manifest, onPhone) { vm.libraryGroups() }

    Column(Modifier.fillMaxSize()) {
        if (selection.isNotEmpty()) {
            Row(Modifier.fillMaxWidth().padding(12.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                val selected = groups.flatMap { it.second }.filter { selection.contains(it.id) }
                if (selected.any { it.itemState is ItemState.OnPhone })
                    Button(onClick = { onTransfer(TransferKind.OFFLOAD, selected); selection.clear() }) { Text("Free up space") }
                if (selected.any { it.itemState is ItemState.OnTucklet })
                    OutlinedButton(onClick = { onTransfer(TransferKind.LOAD, selected); selection.clear() }) { Text("Get a copy") }
            }
        }
        LazyColumn(Modifier.fillMaxSize()) {
            groups.forEach { (app, items) ->
                item { Text(app, fontWeight = FontWeight.Bold, color = Brand.muted, modifier = Modifier.padding(16.dp, 12.dp, 16.dp, 4.dp)) }
                items(items, key = { it.id }) { item ->
                    LibraryRow(vm, item, selected = selection.contains(item.id),
                        onToggle = { if (selection.contains(item.id)) selection.remove(item.id) else selection.add(item.id) },
                        onOpen = { onOpen(item) })
                }
            }
        }
    }
}

@Composable
private fun LibraryRow(
    vm: AppViewModel, item: MediaItem, selected: Boolean,
    onToggle: () -> Unit, onOpen: () -> Unit,
) {
    Row(Modifier.fillMaxWidth().padding(horizontal = 12.dp, vertical = 6.dp), verticalAlignment = Alignment.CenterVertically) {
        IconButton(onClick = onToggle) {
            Icon(if (selected) Icons.Filled.CheckCircle else Icons.Filled.RadioButtonUnchecked, "select",
                tint = if (selected) Brand.accent else Brand.muted)
        }
        Thumb(vm, item, 44)
        Spacer(Modifier.width(12.dp))
        Column(Modifier.weight(1f).clickable { onOpen() }) {
            Text(item.name, color = Brand.ink, maxLines = 1)
            Row(verticalAlignment = Alignment.CenterVertically) {
                StateChip(item.itemState)
                Spacer(Modifier.width(8.dp))
                Text(vm.byteFormat(item.sizeBytes), fontSize = 12.sp, color = Brand.muted)
            }
        }
        Icon(Icons.Filled.ChevronRight, null, tint = Brand.muted)
    }
}

@Composable
private fun Thumb(vm: AppViewModel, item: MediaItem, size: Int) {
    var bmp by remember(item.id) { mutableStateOf<Bitmap?>(null) }
    LaunchedEffect(item.id) { bmp = vm.thumbnail(item) }
    Box(Modifier.size(size.dp).clip(RoundedCornerShape(8.dp)).background(Brand.accent.copy(alpha = 0.10f)),
        contentAlignment = Alignment.Center) {
        val b = bmp
        if (b != null) Image(b.asImageBitmap(), null, modifier = Modifier.fillMaxSize())
        else Icon(if (item.isVideo) Icons.Filled.Videocam else Icons.Filled.Photo, null, tint = Brand.muted)
    }
}

@Composable
private fun StateChip(state: ItemState) {
    val (icon, label) = when (state) {
        is ItemState.OnPhone -> Icons.Filled.Smartphone to "On phone"
        is ItemState.OnTucklet -> Icons.Filled.Album to "On Tucklet"
        is ItemState.Temporary -> Icons.Filled.Schedule to "Temporary"
    }
    Row(verticalAlignment = Alignment.CenterVertically) {
        Icon(icon, null, tint = Brand.accent, modifier = Modifier.size(14.dp))
        Spacer(Modifier.width(4.dp))
        Text(label, fontSize = 12.sp, color = Brand.muted)
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun ItemDetailSheet(vm: AppViewModel, item: MediaItem, onClose: () -> Unit) {
    ModalBottomSheet(onDismissRequest = onClose, containerColor = Brand.paper) {
        Column(Modifier.padding(20.dp).fillMaxWidth(), horizontalAlignment = Alignment.CenterHorizontally) {
            Thumb(vm, item, 200)
            Spacer(Modifier.height(12.dp))
            Text(item.name, fontWeight = FontWeight.Bold, color = Brand.ink)
            StateChip(item.itemState)
            Spacer(Modifier.height(8.dp))
            Text("From ${item.origin.app}", color = Brand.muted)
            Spacer(Modifier.height(16.dp))
            when (item.itemState) {
                is ItemState.OnTucklet -> {
                    Button(onClick = { vm.restore(item); onClose() }, modifier = Modifier.fillMaxWidth()) { Text("Put back on phone") }
                    Spacer(Modifier.height(8.dp))
                    OutlinedButton(onClick = { vm.delete(item); onClose() }, modifier = Modifier.fillMaxWidth()) { Text("Delete from Tucklet") }
                }
                is ItemState.OnPhone ->
                    Button(onClick = { vm.runOffload(listOf(item)) { onClose() } }, modifier = Modifier.fillMaxWidth()) { Text("Back up to Tucklet") }
                is ItemState.Temporary -> Text("This is a temporary copy on your phone.", color = Brand.muted)
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun TransferSheet(vm: AppViewModel, kind: TransferKind, items: List<MediaItem>, onClose: () -> Unit) {
    var policy by remember { mutableStateOf(TemporaryPolicy.ONE_WEEK) }
    val est = remember(items) { vm.estimate(items) }
    ModalBottomSheet(onDismissRequest = onClose, containerColor = Brand.paper) {
        Column(Modifier.padding(20.dp).fillMaxWidth(), horizontalAlignment = Alignment.CenterHorizontally) {
            Text(if (kind == TransferKind.OFFLOAD) "Free up space" else "Get a copy",
                fontSize = 22.sp, fontWeight = FontWeight.Bold, color = Brand.ink)
            Text("${items.size} items · ${vm.byteFormat(est.bytesTotal)}", color = Brand.muted)
            Spacer(Modifier.height(16.dp))
            Card(Modifier.fillMaxWidth()) { Column(Modifier.padding(20.dp), horizontalAlignment = Alignment.CenterHorizontally) {
                Text("About ${est.human}", fontSize = 32.sp, fontWeight = FontWeight.Bold, color = Brand.accent)
                Text("over Wi-Fi", fontSize = 12.sp, color = Brand.muted)
            } }
            if (kind == TransferKind.LOAD) {
                Spacer(Modifier.height(12.dp))
                Text("Keep on phone for", color = Brand.ink)
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    TemporaryPolicy.entries.forEach { p ->
                        FilterChip(selected = policy == p, onClick = { policy = p }, label = { Text(p.label) })
                    }
                }
            }
            Spacer(Modifier.height(20.dp))
            Button(
                onClick = {
                    if (kind == TransferKind.OFFLOAD) vm.runOffload(items) { onClose() }
                    else vm.runLoad(items, policy) { onClose() }
                },
                modifier = Modifier.fillMaxWidth(),
            ) { Text("Start") }
            Spacer(Modifier.height(8.dp))
        }
    }
}

@Composable
private fun SettingsScreen(vm: AppViewModel) {
    val context = LocalContext.current
    var auto by remember { mutableStateOf(true) }
    val status by vm.status.collectAsState()
    val caps by vm.capabilities.collectAsState()
    val lastOffload by vm.lastOffloadIds.collectAsState()

    LazyColumn(Modifier.fillMaxSize().padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
        item {
            Card { Column(Modifier.padding(16.dp)) {
                Text("Backup", fontWeight = FontWeight.Bold, color = Brand.ink)
                Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                    Text("Back up automatically", Modifier.weight(1f), color = Brand.ink)
                    Switch(checked = auto, onCheckedChange = {
                        auto = it
                        if (it) vm.enableTrickle(context) else vm.disableTrickle(context)
                    })
                }
            } }
        }
        item {
            Card { Column(Modifier.padding(16.dp)) {
                Text("Device", fontWeight = FontWeight.Bold, color = Brand.ink)
                Text("Storage: ${caps?.storage?.label ?: "—"}", color = Brand.muted)
                status?.let {
                    Text("Battery: ${it.batteryPercent}%", color = Brand.muted)
                    Text("Firmware: ${it.firmwareVersion}", color = Brand.muted)
                }
            } }
        }
        if (lastOffload.isNotEmpty()) {
            item { OutlinedButton(onClick = { vm.undoLastOffload() }, modifier = Modifier.fillMaxWidth()) {
                Text("Undo last backup (${lastOffload.size} photos)")
            } }
        }
        item {
            Card { Column(Modifier.padding(16.dp)) {
                Text("Paired phones", fontWeight = FontWeight.Bold, color = Brand.ink)
                Text("This phone", color = Brand.ink)
                Spacer(Modifier.height(8.dp))
                TextButton(onClick = { vm.forget() }) { Text("Forget this Tucklet", color = MaterialTheme.colorScheme.error) }
                Text(
                    "Forgetting stops this phone from connecting. To also erase this phone from the Tucklet itself, hold its button for 5 seconds to factory-reset it.",
                    fontSize = 12.sp, color = Brand.muted,
                )
            } }
        }
    }
}
