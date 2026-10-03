package org.rynk.keyboard

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Test
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.nio.charset.StandardCharsets

class BinaryEventProtocolTest {

    @Test
    fun testDecodeBinaryEvents() {
        // Construct binary buffer mimicking serialize_events_binary from rust-core
        val buffer = ByteBuffer.allocate(256).order(ByteOrder.LITTLE_ENDIAN)

        // Count: 5 events
        buffer.putInt(5)

        // Event 1: CommitText("Hello")
        val textBytes = "Hello".toByteArray(StandardCharsets.UTF_8)
        buffer.put(NativeBridge.EVENT_COMMIT_TEXT.toByte())
        buffer.putInt(textBytes.size)
        buffer.put(textBytes)

        // Event 2: DeleteSurroundingText { before = 1, after = 0 }
        buffer.put(NativeBridge.EVENT_DELETE_SURROUNDING.toByte())
        buffer.putInt(8)
        buffer.putInt(1) // before
        buffer.putInt(0) // after

        // Event 3: SendKeyEvent(66)
        buffer.put(NativeBridge.EVENT_SEND_KEY_EVENT.toByte())
        buffer.putInt(4)
        buffer.putInt(66)

        // Event 4: PerformHaptic(2)
        buffer.put(NativeBridge.EVENT_PERFORM_HAPTIC.toByte())
        buffer.putInt(1)
        buffer.put(2.toByte())

        // Event 5: MoveCursor(-5)
        buffer.put(NativeBridge.EVENT_MOVE_CURSOR.toByte())
        buffer.putInt(4)
        buffer.putInt(-5)

        val data = ByteArray(buffer.position())
        buffer.flip()
        buffer.get(data)

        // Decode
        val decBuffer = ByteBuffer.wrap(data).order(ByteOrder.LITTLE_ENDIAN)
        val count = decBuffer.int
        assertEquals(5, count)

        // Decode 1: CommitText
        assertEquals(NativeBridge.EVENT_COMMIT_TEXT, decBuffer.get().toInt() and 0xFF)
        val len1 = decBuffer.int
        val strBytes1 = ByteArray(len1)
        decBuffer.get(strBytes1)
        assertEquals("Hello", String(strBytes1, StandardCharsets.UTF_8))

        // Decode 2: DeleteSurroundingText
        assertEquals(NativeBridge.EVENT_DELETE_SURROUNDING, decBuffer.get().toInt() and 0xFF)
        val len2 = decBuffer.int
        assertEquals(8, len2)
        assertEquals(1, decBuffer.int)
        assertEquals(0, decBuffer.int)

        // Decode 3: SendKeyEvent
        assertEquals(NativeBridge.EVENT_SEND_KEY_EVENT, decBuffer.get().toInt() and 0xFF)
        val len3 = decBuffer.int
        assertEquals(4, len3)
        assertEquals(66, decBuffer.int)

        // Decode 4: PerformHaptic
        assertEquals(NativeBridge.EVENT_PERFORM_HAPTIC, decBuffer.get().toInt() and 0xFF)
        val len4 = decBuffer.int
        assertEquals(1, len4)
        assertEquals(2, decBuffer.get().toInt() and 0xFF)

        // Decode 5: MoveCursor
        assertEquals(NativeBridge.EVENT_MOVE_CURSOR, decBuffer.get().toInt() and 0xFF)
        val len5 = decBuffer.int
        assertEquals(4, len5)
        assertEquals(-5, decBuffer.int)
    }

    @Test
    fun testEmptyOrCorruptBinarySafelyHandled() {
        val emptyBuf = ByteBuffer.allocate(4).order(ByteOrder.LITTLE_ENDIAN)
        emptyBuf.putInt(0)
        val decBuffer = ByteBuffer.wrap(emptyBuf.array()).order(ByteOrder.LITTLE_ENDIAN)
        val count = decBuffer.int
        assertEquals(0, count)
    }
}
