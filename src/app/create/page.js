"use client";

import React, { useState, useEffect } from "react";
import { Input, Button, CircularProgress, Modal, ModalContent, ModalHeader, ModalBody, ModalFooter } from "@nextui-org/react";
import { generateUsername } from "unique-username-generator";
import { TfiReload } from "react-icons/tfi";
import { invoke } from "@tauri-apps/api/tauri";
import BackArrow from "@/components/BackArrow";

export default function CreateChat() {
    const [randomName, setRandomName] = useState("");
    const [username, setUsername] = useState("");
    const [usernameError, setUsernameError] = useState(false);
    const [limit, setLimit] = useState(2);
    const [loading, setLoading] = useState(false);
    const [modalError, setModalError] = useState(false);
    const [error, setError] = useState("");

    useEffect(() => {
        const name = generateUsername("", 5);
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
        const newName = generateUsername("", 5);
        setRandomName(newName);
        setUsername(newName);
    };

    const handleLimitChange = (e) => {
        const newLimit = parseInt(e.target.value) || 2;
        setLimit(newLimit);
    };

    const handleSubmit = () => {
        setLoading(true)
        invoke('create_chat', { username: username, userLimit: limit }).then(() => {
            console.log("submitted");
            setLoading(false)
        }).catch((err) => {
            setError(err)
            setModalError(true)
        });
    };

    return (
        <main>
            <BackArrow location={"/"} />
            <div className="h-[90vh] flex justify-center items-center flex-col">
                <h1 className="font-bold text-5xl mt-0">Create a Chat Room</h1>
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
                <Input 
                    size="lg"
                    label="Chat Limit"
                    variant="faded"
                    description="How many users can join"
                    className="max-w-[40vw] mt-10"
                    placeholder="2"
                    type="number"
                    value={limit}
                    onChange={handleLimitChange}
                />
                <Button color="primary" className="mt-3 pr-10 pl-10 pt-6 pb-6 font-bold" onClick={handleSubmit}>
                    {
                        loading ?
                        <CircularProgress color="success"/> :
                        "Create"
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
