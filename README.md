Installation and Run:

1. Install Rust in https://www.rust-lang.org/
2. Open terminal in code editor (VsCode) and type "git clone https://github.com/Almons65/Disk_Usage_Visualizer.git"
3. cd to the folder "Disk_Usage_Visualizer"
4. type "cargo run" in terminal

Done!



What is Disk Usage Visualizer?

A software tool designed to analyze and display how storage space is utilized on a computer's disk. It scans the file system and presents a visual representation of the disk’s contents, allowing users to easily see how much space is being taken up by files and directories.




Main Features:

- Visualization of Disk Space: It provides a graphical interface to represent disk space usage, often showing total, used, and free space on each disk. This helps users quickly identify storage utilization.
  
- Progress Tracking: As the disk scan progresses, users can see real-time feedback, often through progress bars or similar indicators, showing the status of the scan.
  
- Performance Optimization: Disk Usage Visualizers are designed to efficiently handle large amounts of data, even when scanning large disks with numerous files and directories. They often utilize multithreading or parallel processing to speed up the scanning process.
  
- Search by File Name: Users can input part or all of a file’s name, and the tool will locate and display the relevant file or folder, reducing the time spent looking through large directories.
  
- Search by File Type: The search function can filter files based on their extensions (e.g., .txt, .jpg, .mp4), making it easy to find files of a specific type.




How to use the program:

Click the “Scan Disk” button.
- The program will automatically start scanning the disks mounted on your system.
- A progress bar will indicate the real-time progress of the scan.
Once the scan is complete, the program will display the following information for each disk:
- Disk Name: The name or identifier of the disk.
- Total Space: The total available storage on the disk (in GB).
- Used Space: The amount of storage currently in use (in GB).
- Total Files: The total number of files stored on the disk.
- Total File Size: The cumulative size of all files (in GB).
If you want to perform a new scan, click the “Refresh Disk Info” button.
- This will trigger a fresh scan of the disk usage
After the scan is complete, the time taken to perform the scan is displayed at the bottom of the interface (e.g., "Scan Time: X.XX seconds").







Screenshots of the program:

![image](https://github.com/user-attachments/assets/ec9e9437-a441-4d65-87d2-152458116856)


![image](https://github.com/user-attachments/assets/daa37eff-4516-4753-b8a2-7fea3c8959d0)


![image](https://github.com/user-attachments/assets/ee84bc52-accd-47ec-8cc5-5d182cd55386)
