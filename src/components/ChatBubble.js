export default function ChatBubble({ author, time, content, self }) {
    return (
        <div className={`flex items-start gap-x-2.5 mt-5 mb-5 ${self ? 'justify-end' : 'justify-start'}`}>
            <div class={`flex flex-col w-full max-w-[250px] leading-1.5 p-4 ${self ? "bg-primary rounded-s-xl rounded-br-xl" : "bg-[#454545] rounded-e-xl rounded-es-xl"}`}>
                <div class="flex items-center space-x-2 rtl:space-x-reverse">
                    <span class="text-sm font-semibold text-white">{author}</span>
                    <span class="text-sm font-normal text-gray-400">{time}</span>
                </div>
                <p class="text-sm font-normal text-white">{content}</p>
            </div>
        </div>
    )
}