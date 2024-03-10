"use client";

import React, { useState, useEffect } from "react";
import { Input, Button, CircularProgress, Modal, ModalContent, ModalHeader, ModalBody, ModalFooter } from "@nextui-org/react";
import { generateUsername } from "unique-username-generator";
import { TfiReload } from "react-icons/tfi";
import { invoke } from "@tauri-apps/api/tauri";
import BackArrow from "@/components/BackArrow";
import { useSearchParams } from "next/navigation";
import { FaLink } from "react-icons/fa";
import { RiLockPasswordLine } from "react-icons/ri";

export default function Handle() {
    const params = useSearchParams();
    const isCreate = params.get("type") === "create";

    const url_regex = new RegExp("(temp?:\/\/[a-f0-9]*_{1}[a-f0-9]*)")
    const [invalid, setInvalid] = useState(false);
    const [url, setUrl] = useState("");

    const [randomName, setRandomName] = useState("");
    const [username, setUsername] = useState("");
    const [usernameError, setUsernameError] = useState(false);

    const [limit, setLimit] = useState(1);
    const [loading, setLoading] = useState(false);
    const [modalError, setModalError] = useState(false);
    const [error, setError] = useState("");

    const [password, setPassword] = useState("");
    const [passwordInvalid, setPasswordInvalid] = useState(false);

    useEffect(() => {
        const name = generateUsername("", 3, 15);
        setRandomName(name);
        setUsername(name);
    }, []);

    const handleUsernameChange = (e) => {
        const newUsername = e.target.value;
        if (newUsername.length > 15) {
            setUsernameError(true);
            return;
        }
        if (usernameError) {
            setUsernameError(false);
        }
        setUsername(newUsername || randomName);
    };

    const generateRandomName = () => {
        const newName = generateUsername("", 3, 15);
        setRandomName(newName);
        setUsername(newName);
    };

    const handleLimitChange = (e) => {
        const newLimit = parseInt(e.target.value) || 1;
        setLimit(newLimit);
    };

    const handleCreate = () => {
        if(password.length === 0) {
            setPasswordInvalid(true)
            return
        }
        setPasswordInvalid(false)
        setLoading(true)
        invoke('create_chat', { username: username, userLimit: limit, password: password }).then((url) => {
            setLoading(false);
            window.location.href = `/chat?roomURL=${url}&username=${username}&type=host`
        }).catch((err) => {
            setError(err)
            setModalError(true)
        });
    };

    const handleJoin = () => { //TODO: Check for custom join URL schema
        if(url.length === 0 || url.includes(" ") || !url_regex.test(url)) {
            setInvalid(true)
            return
        }
        setInvalid(false)
        if(password.length === 0) {
            setPasswordInvalid(true)
            return
        }
        setPasswordInvalid(false)
        setLoading(true)
        invoke('join_chat', { username: username, chatUrl: url, password: password }).then((e) => {
            setLoading(false)
            window.location.href = `/chat?roomURL=${url}&username=${username}&type=client`
        }).catch((err) => {
            setError(err)
            setModalError(true)
        })
    }

    return (
        <main>
            <BackArrow location={"/"} />
            <div className="h-[90vh] flex justify-center items-center flex-col">
                <h1 className="font-bold text-5xl mt-0">
                    { isCreate ? "Create a Chat Room" : "Join a Chat Room" }
                </h1>
                <Input  
                    size="lg"
                    label="Username"
                    variant="faded"
                    description="If empty, random username will be assigned"
                    placeholder={randomName}
                    className="max-w-[40vw] mt-10"
                    isInvalid={usernameError}
                    onChange={handleUsernameChange}
                    endContent={
                        <Button className="w-fit" onClick={generateRandomName}>
                            <TfiReload color="white" size={"20px"}/>
                        </Button>
                    }
                />
                {
                    isCreate ?
                    <Input 
                        size="lg"
                        label="Chat Limit"
                        variant="faded"
                        description="How many users can join (besides the host)"
                        className="max-w-[40vw] mt-10"
                        placeholder="1"
                        type="number"
                        value={limit}
                        onChange={handleLimitChange}
                    /> :
                    <Input  
                        size="lg"
                        label="Join Url"
                        variant="faded"
                        description="The chat room url"
                        className="max-w-[40vw] mt-10"
                        startContent={<FaLink color="white" />}
                        isInvalid={invalid}
                        onChange={(e) => setUrl(e.currentTarget.value)}
                    />
                }
                <Input  
                    type="password"
                    size="lg"
                    label="Chat Password"
                    variant="faded"
                    description="The chat room password"
                    className="max-w-[40vw] mt-10"
                    startContent={<RiLockPasswordLine color="white" />}
                    isInvalid={passwordInvalid}
                    onChange={(e) => setPassword(e.currentTarget.value)}
                />
                <Button color="primary" className="mt-3 pr-10 pl-10 pt-6 pb-6 font-bold" onClick={isCreate ? handleCreate : handleJoin}>
                    {
                        loading ?
                        <CircularProgress color="success"/> :
                        (
                            isCreate ? "Create" : "Join"
                        )
                    }
                </Button>
                <Modal isOpen={modalError}>
                    <ModalContent>
                        <ModalHeader>An error occured...</ModalHeader>
                        <ModalBody>
                            <h1>{error}</h1>
                        </ModalBody>
                        <ModalFooter>
                            <Button color="danger" onPress={() => { setModalError(false); setLoading(false) }}>
                                Close
                            </Button>
                        </ModalFooter>
                    </ModalContent>
                </Modal>
            </div>
            
        </main>
    );
}
