package dms.application.fake.system

import dms.application.port.system.IdGenerator

/** Ids `id-1`, `id-2`, … so tests can predict them. */
class SequentialIds : IdGenerator {

    private var next = 1

    override fun newId(): String {
        return "id-${next++}"
    }
}
