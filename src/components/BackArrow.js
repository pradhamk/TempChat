"use client";

import { Link } from "@nextui-org/react";
import { IoIosArrowBack } from "react-icons/io";

export default function BackArrow({ location }) {
    return (
        <Link href={location}>
            <IoIosArrowBack color="white" className="size-6 absolute mt-5 ml-5"/>
        </Link>
    )
}