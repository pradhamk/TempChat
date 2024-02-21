export default function JoinLeave({ username, isJoin }) {
    return (
        <div className="flex items-center">
            <div className="flex-1 border-t-2 border-gray-700"></div>
            <h1 className="px-3 text-gray-400">{username} { isJoin ? "joined" : "left" }</h1>
            <div className="flex-1 border-t-2 border-gray-700"></div>
        </div>
    )
}