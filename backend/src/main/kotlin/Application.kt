package blazern

import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.main
import com.github.ajalt.clikt.parameters.options.help
import com.github.ajalt.clikt.parameters.options.option
import com.github.ajalt.clikt.parameters.options.required
import io.ktor.server.application.*

class BackendCmd(
    private val args: Array<String>,
) : CliktCommand() {
    val apiKeyChatGpt: String by option()
        .required()
        .help("ChatGPT API key to send requests to ChatGPT")

    override fun run() {
        io.ktor.server.netty.EngineMain.main(args)
    }
}

fun main(args: Array<String>) = BackendCmd(args).main(args)

fun Application.module() {
    configureRouting()
}
