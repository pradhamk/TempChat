"use client";

import { useSearchParams } from "next/navigation";
import { Divider, Button, Textarea, Modal, ModalBody, ModalContent, ModalHeader, ModalFooter } from "@nextui-org/react";
import { FaRegCopy } from "react-icons/fa";
import { IoMdSend } from "react-icons/io";
import { useRef, useState } from "react";
import { IoMdClose } from "react-icons/io";
import ChatBubble from "@/components/ChatBubble";
import { listen, emit } from "@tauri-apps/api/event";
import { useEffect } from "react";
import JoinLeave from "@/components/JoinLeave";

export default function ChatRoom() {
    const search = useSearchParams()
    const room_url = search.get('roomURL')
    const username = search.get('username')
    const isHost = search.get("type") === "host"

    const [message, setMessage] = useState("")
    const [messages, setMessages] = useState([])
    const [isShutdown, setShutdown] = useState(false)
    const msgRef = useRef(null)

    function sendMessage(e) {
        if(message.length === 0) {
            return
        }

        emit("host-message", { content: message })
            .then(() => {
                setMessage("")
            }).catch((e) => {
                console.log("Couldn't send message", e)
            })  
    }

    useEffect(() => {
        if(msgRef.current) {
            msgRef.current.scrollTop = msgRef.current.scrollHeight;
        }
    }, [messages])

    useEffect(() => {
        if(!window) { return }
        const message_unlisten = listen('new-message', (e) => {
            const content = JSON.parse(e.payload)
            console.log(content)
            setMessages((prev) => [...prev, content])
        })

        const join_unlisten = listen('join', (e) => {
            const content = JSON.parse(e.payload)
            setMessages((prev) => [...prev, content])
        })

        const error_unlisten = listen('error', (e) => {
            console.log("error", e.payload)
        })

        const shutdown_unlisten = listen('shutdown', (e) => {
            setShutdown(true)
        })

        const exit_unlisten = listen('client_exit', (e) => {
            console.log(e.payload)
            const content = JSON.parse(e.payload)
            console.log(content)
            setMessages((prev) => [...prev, { exit: content}])
        })  

        return () => {
            message_unlisten.then(f => f())
            join_unlisten.then(f => f())
            error_unlisten.then(f => f())
            shutdown_unlisten.then(f => f())
            exit_unlisten.then(f => f())
        }
    }, [])

    return (
        <main className="w-full flex flex-col items-center">
            <div className="flex mt-8 mb-3 w-[80vw] items-center">
                <h1 className="text-xl font-bold text-center">Chat ID: 
                    <Button 
                        color="primary" 
                        variant="flat" 
                        className="ml-3 text-white" 
                        startContent={<FaRegCopy color="purple"/>}
                        onClick={() => { navigator.clipboard.writeText(room_url) }}
                    >
                        {room_url.split(".")[0].replace("https://", "")}
                    </Button>
                </h1>
                <Button 
                    color="danger" 
                    variant="ghost" 
                    className="ml-auto" 
                    size="sm"
                    onClick={() => {
                        isHost ?
                        emit("shutdown").then(() => {
                            //Shutdown modal closes
                            window.location.href = "/"
                        }) :
                        emit("client_exit").then(() => {
                            window.location.href = "/"
                        })
                    }}
                >
                    <IoMdClose size={25}/>
                </Button>
            </div>
            
            <Divider className="w-[80vw]"/>
            <div className="w-[80vw] max-h-[80vh] pb-20 mt-3 overflow-y-scroll scroll-smooth" ref={msgRef}>
                {
                    messages.map((val, i) => {
                        if(val.joinMessage) {
                            return (<JoinLeave username={val.joinMessage.joined} key={i} isJoin={true}/>)                            
                        } else if(val.exit) {
                            return (<JoinLeave username={val.exit.username} key={i} isJoin={false}/>)
                        } else {
                            return (<ChatBubble time={val.created} author={val.sender} content={val.content} self={val.sender === username ? true : false} key={i}/>)
                        }
                    })
                }
            </div>
            <div className="flex items-center absolute bottom-0 pb-10 bg-background">
                <Textarea
                    placeholder="Send your message"
                    maxLength={1000}
                    maxRows={4}
                    minRows={1}
                    variant="faded"
                    value={message}
                    onValueChange={setMessage}
                    className="w-[80vw] mr-3"
                    onKeyDown={(e) => {
                        if(e.key === "Enter" && !e.shiftKey) {
                            e.preventDefault();
                            sendMessage()
                        }
                    }}
                />
                <Button color="primary" onClick={sendMessage} onPress={null}>
                    <IoMdSend className="size-5"/>
                </Button>
            </div>
            <Modal
                isOpen={isShutdown}
                isDismissable={false}
            >
                <ModalContent>
                    <ModalHeader>Chat Closed</ModalHeader>
                    <ModalBody>
                        The chat lobby has been closed by the host.
                    </ModalBody>
                    <ModalFooter>
                        <Button 
                            color="primary" 
                            onPress={() => {
                                setShutdown(false)
                                window.location.href = "/"
                            }}
                        >
                            Continue
                        </Button>
                    </ModalFooter>
                </ModalContent>
            </Modal>
        </main>
    )
}